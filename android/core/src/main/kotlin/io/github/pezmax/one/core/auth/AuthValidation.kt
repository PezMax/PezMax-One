package io.github.pezmax.one.core.auth

/**
 * 账户表单校验（spec/api/auth.md、fixtures/auth/validation.json）。
 *
 * 为什么客户端也校验：省一次往返、把错误落到具体字段；代价是文案必须与上游逐字一致。
 * 计数口径尤其不能统一：用户名 / 密保答案按 **rune**，密码 / 头像地址按 **UTF-8 字节**。
 *
 * 这里只返回第一处错误的文案（上游 switch 顺序即契约），不做多错误聚合。
 */
object AuthValidation {

    const val ERR_USERNAME_EMPTY = "用户名不能为空"
    const val ERR_USERNAME_FORMAT = "用户名格式错误"
    const val ERR_PASSWORD_FORMAT = "密码格式错误：长度需在 5 到 64 位之间"
    const val ERR_PASSWORD_MISMATCH = "两次输入的密码不一致"
    const val ERR_QUESTION_EMPTY = "密保问题不能为空"
    const val ERR_ANSWER_EMPTY = "密保答案不能为空"
    const val ERR_AVATAR_FORMAT = "头像地址格式错误"
    const val ERR_CAPTCHA = "验证码错误或已过期"
    const val ERR_CREDENTIALS = "用户名或密码错误"

    const val MAX_USERNAME_RUNES = 64
    const val MIN_PASSWORD_BYTES = 5
    const val MAX_PASSWORD_BYTES = 64
    const val MAX_AVATAR_BYTES = 255

    /** Go 的 `TrimSpace` 覆盖 Unicode 空白；Kotlin 的 `isWhitespace()` 与之足够接近。 */
    fun trimSpace(value: String): String = value.trim()

    /** Go 的 `len([]rune(s))`：按 Unicode 码点计数，代理对算一个。 */
    fun runeLength(value: String): Int = value.codePointCount(0, value.length)

    fun utf8Length(value: String): Int = value.toByteArray(Charsets.UTF_8).size

    /** 返回 null 表示通过；否则是必须原样展示给用户的上游文案。 */
    fun validateRegister(request: RegisterDraft): String? {
        val username = trimSpace(request.username)
        if (username.isEmpty()) return ERR_USERNAME_EMPTY
        if (runeLength(username) > MAX_USERNAME_RUNES) return ERR_USERNAME_FORMAT
        validatePasswordPair(request.password, request.confirmPassword)?.let { return it }
        val questions = listOf(request.securityQuestionOne, request.securityQuestionTwo, request.securityQuestionThree)
        if (questions.any { trimSpace(it).isEmpty() }) return ERR_QUESTION_EMPTY
        val answers = listOf(request.securityAnswerOne, request.securityAnswerTwo, request.securityAnswerThree)
        if (answers.any { trimSpace(it).isEmpty() }) return ERR_ANSWER_EMPTY
        if (utf8Length(request.avatar) > MAX_AVATAR_BYTES) return ERR_AVATAR_FORMAT
        if (request.code.isBlank() || request.uuid.isBlank()) return ERR_CAPTCHA
        return null
    }

    fun validateReset(request: ResetDraft): String? {
        // 第一步签发的工单键是 uuid + username；缺任何一项都按验证码失效处理。
        if (trimSpace(request.username).isEmpty() || request.uuid.isBlank() || request.code.isBlank()) {
            return ERR_CAPTCHA
        }
        val answers = listOf(request.securityAnswerOne, request.securityAnswerTwo, request.securityAnswerThree)
        if (answers.any { trimSpace(it).isEmpty() }) return ERR_ANSWER_EMPTY
        validatePasswordPair(request.newPassword, request.confirmPassword)?.let { return it }
        return null
    }

    fun validateLogin(request: LoginDraft): String? =
        if (trimSpace(request.username).isEmpty() || request.password.isEmpty()) ERR_CREDENTIALS else null

    private fun validatePasswordPair(password: String, confirm: String): String? {
        val bytes = utf8Length(password)
        if (bytes < MIN_PASSWORD_BYTES || bytes > MAX_PASSWORD_BYTES) return ERR_PASSWORD_FORMAT
        if (password != confirm) return ERR_PASSWORD_MISMATCH
        return null
    }
}

data class RegisterDraft(
    val username: String = "",
    val password: String = "",
    val confirmPassword: String = "",
    val securityQuestionOne: String = "",
    val securityQuestionTwo: String = "",
    val securityQuestionThree: String = "",
    val securityAnswerOne: String = "",
    val securityAnswerTwo: String = "",
    val securityAnswerThree: String = "",
    val avatar: String = "",
    val code: String = "",
    val uuid: String = "",
)

data class ResetDraft(
    val username: String = "",
    val code: String = "",
    val uuid: String = "",
    val securityAnswerOne: String = "",
    val securityAnswerTwo: String = "",
    val securityAnswerThree: String = "",
    val newPassword: String = "",
    val confirmPassword: String = "",
)

data class LoginDraft(
    val username: String = "",
    val password: String = "",
)
