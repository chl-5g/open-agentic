package ai.openagentic.app

import android.content.Context
import android.content.SharedPreferences
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import ai.openagentic.app.api.ApiClient
import ai.openagentic.app.api.LoginRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch

data class ChatMessage(
    val content: String,
    val isUser: Boolean,
    val isStreaming: Boolean = false,
)

data class ChatUiState(
    val messages: List<ChatMessage> = emptyList(),
    val isLoading: Boolean = false,
    val isConnected: Boolean = false,
    val errorMessage: String? = null,
    val gatewayUrl: String = "",
    val username: String = "",
    val password: String = "",
    val token: String? = null,
)

class ChatViewModel : ViewModel() {

    private val _uiState = MutableStateFlow(ChatUiState())
    val uiState: StateFlow<ChatUiState> = _uiState.asStateFlow()

    private var apiClient: ApiClient? = null
    private var prefs: SharedPreferences? = null

    fun init(context: Context) {
        prefs = context.getSharedPreferences("openagentic", Context.MODE_PRIVATE)
        val saved = prefs!!
        _uiState.value = _uiState.value.copy(
            gatewayUrl = saved.getString("gateway_url", "http://192.168.0.15:18789") ?: "",
            username = saved.getString("username", "admin") ?: "",
            password = saved.getString("password", "") ?: "",
        )
        // Auto-connect if we have settings
        if (_uiState.value.gatewayUrl.isNotEmpty()) {
            connect()
        }
    }

    fun updateSettings(gatewayUrl: String, username: String, password: String) {
        _uiState.value = _uiState.value.copy(
            gatewayUrl = gatewayUrl,
            username = username,
            password = password,
        )
        prefs?.edit()
            ?.putString("gateway_url", gatewayUrl)
            ?.putString("username", username)
            ?.putString("password", password)
            ?.apply()
    }

    fun connect() {
        val url = _uiState.value.gatewayUrl
        if (url.isBlank()) {
            _uiState.value = _uiState.value.copy(errorMessage = "Please configure Gateway URL")
            return
        }

        apiClient = ApiClient(url)
        _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)

        viewModelScope.launch {
            try {
                // Health check
                apiClient!!.api.health()

                // Login if credentials provided
                val user = _uiState.value.username
                val pass = _uiState.value.password
                if (user.isNotBlank() && pass.isNotBlank()) {
                    val loginResp = apiClient!!.api.login(LoginRequest(user, pass))
                    _uiState.value = _uiState.value.copy(
                        isConnected = true,
                        isLoading = false,
                        token = loginResp.token,
                        errorMessage = null,
                    )
                } else {
                    _uiState.value = _uiState.value.copy(
                        isConnected = true,
                        isLoading = false,
                        errorMessage = null,
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isConnected = false,
                    isLoading = false,
                    errorMessage = "Connection failed: ${e.message}",
                )
            }
        }
    }

    fun sendMessage(text: String) {
        if (text.isBlank()) return

        val client = apiClient ?: return
        val token = _uiState.value.token

        // Add user message
        val userMsg = ChatMessage(content = text, isUser = true)
        _uiState.value = _uiState.value.copy(
            messages = _uiState.value.messages + userMsg,
            isLoading = true,
        )

        viewModelScope.launch {
            try {
                if (token != null) {
                    // Try streaming first
                    val aiMsg = ChatMessage(content = "", isUser = false, isStreaming = true)
                    _uiState.value = _uiState.value.copy(
                        messages = _uiState.value.messages + aiMsg,
                    )

                    val sb = StringBuilder()
                    client.chatStream(_uiState.value.gatewayUrl, token, text)
                        .catch { e ->
                            // Fallback to non-streaming
                            val resp = client.chat(token, text)
                            val content = resp.response ?: resp.error ?: "No response"
                            updateLastAiMessage(content, streaming = false)
                        }
                        .collect { chunk ->
                            sb.append(chunk)
                            updateLastAiMessage(sb.toString(), streaming = true)
                        }
                    // Mark streaming done
                    updateLastAiMessage(sb.toString(), streaming = false)
                } else {
                    // No token, try without auth
                    val resp = client.chat("", text)
                    val content = resp.response ?: resp.error ?: "No response"
                    val aiMsg = ChatMessage(content = content, isUser = false)
                    _uiState.value = _uiState.value.copy(
                        messages = _uiState.value.messages + aiMsg,
                    )
                }
            } catch (e: Exception) {
                val errMsg = ChatMessage(
                    content = "Error: ${e.message}",
                    isUser = false,
                )
                _uiState.value = _uiState.value.copy(
                    messages = _uiState.value.messages + errMsg,
                )
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    private fun updateLastAiMessage(content: String, streaming: Boolean) {
        val msgs = _uiState.value.messages.toMutableList()
        if (msgs.isNotEmpty() && !msgs.last().isUser) {
            msgs[msgs.lastIndex] = msgs.last().copy(content = content, isStreaming = streaming)
            _uiState.value = _uiState.value.copy(messages = msgs)
        }
    }

    fun clearMessages() {
        _uiState.value = _uiState.value.copy(messages = emptyList())
    }

    fun dismissError() {
        _uiState.value = _uiState.value.copy(errorMessage = null)
    }
}
