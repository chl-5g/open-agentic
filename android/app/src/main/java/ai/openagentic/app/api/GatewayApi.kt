package ai.openagentic.app.api

import com.google.gson.annotations.SerializedName
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.POST

data class LoginRequest(
    val username: String,
    val password: String,
)

data class LoginResponse(
    val token: String,
    @SerializedName("expires_in") val expiresIn: Long,
    @SerializedName("token_type") val tokenType: String,
)

data class ChatRequest(
    val message: String,
    val model: String? = null,
)

data class ChatResponse(
    val response: String? = null,
    val error: String? = null,
)

data class HealthResponse(
    val status: String,
    val version: String? = null,
)

interface GatewayApi {

    @POST("api/auth/login")
    suspend fun login(@Body request: LoginRequest): LoginResponse

    @GET("health")
    suspend fun health(): HealthResponse

    @POST("chat")
    suspend fun chat(
        @Header("Authorization") auth: String,
        @Body request: ChatRequest,
    ): ChatResponse
}
