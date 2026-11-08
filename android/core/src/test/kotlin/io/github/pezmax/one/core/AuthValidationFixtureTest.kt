package io.github.pezmax.one.core

import io.github.pezmax.one.core.auth.AuthValidation
import io.github.pezmax.one.core.auth.LoginDraft
import io.github.pezmax.one.core.auth.RegisterDraft
import io.github.pezmax.one.core.auth.ResetDraft
import io.github.pezmax.one.core.SpecFixtures.expect
import io.github.pezmax.one.core.SpecFixtures.input
import io.github.pezmax.one.core.json.objectOrNull
import io.github.pezmax.one.core.json.stringOrNull
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** spec/fixtures/auth/validation.json —— 表单校验矩阵，文案必须逐字相等。 */
class AuthValidationFixtureTest {

    @Test
    fun `注册 找回 登录三组用例全部与上游文案一致`() {
        val cases = SpecFixtures.cases("fixtures/auth/validation.json")
        assertTrue(cases.size >= 25)
        cases.forEach { case ->
            val input = case.input()
            val actual = when (input.str("action")) {
                "register" -> AuthValidation.validateRegister(input.toRegister())
                "reset" -> AuthValidation.validateReset(input.toReset())
                "login" -> AuthValidation.validateLogin(LoginDraft(input.str("username"), input.str("password")))
                else -> error("未知 action: ${input.str("action")} in ${case.str("id")}")
            }
            val expected = case.expect()["error"].let { if (it == null || it is JsonNull) null else (it as kotlinx.serialization.json.JsonPrimitive).content }
            assertEquals(case.str("id"), expected, actual)
        }
    }

    @Test
    fun `计数口径不能统一`() {
        // 4 个汉字 = 4 rune、12 字节：按密码的下限 5 字节通过。
        assertEquals(null, AuthValidation.validateRegister(registerDraft(password = "密码两字")))
        // 65 个汉字用户名 = 65 rune，超 64。
        assertEquals(
            AuthValidation.ERR_USERNAME_FORMAT,
            AuthValidation.validateRegister(registerDraft(username = "中".repeat(65))),
        )
        assertEquals(64, AuthValidation.runeLength("中".repeat(64)))
        assertEquals(192, AuthValidation.utf8Length("中".repeat(64)))
    }

    private fun registerDraft(
        username: String = "alice",
        password: String = "12345",
    ) = RegisterDraft(
        username = username,
        password = password,
        confirmPassword = password,
        securityQuestionOne = "q1",
        securityQuestionTwo = "q2",
        securityQuestionThree = "q3",
        securityAnswerOne = "a1",
        securityAnswerTwo = "a2",
        securityAnswerThree = "a3",
        code = "123456",
        uuid = "uuid-fixture",
    )

    private fun JsonObject.toRegister() = RegisterDraft(
        username = str("username"),
        password = str("password"),
        confirmPassword = str("confirmPassword"),
        securityQuestionOne = str("securityQuestionOne"),
        securityQuestionTwo = str("securityQuestionTwo"),
        securityQuestionThree = str("securityQuestionThree"),
        securityAnswerOne = str("securityAnswerOne"),
        securityAnswerTwo = str("securityAnswerTwo"),
        securityAnswerThree = str("securityAnswerThree"),
        avatar = str("avatar"),
        code = str("code"),
        uuid = str("uuid"),
    )

    private fun JsonObject.toReset() = ResetDraft(
        username = str("username"),
        code = str("code"),
        uuid = str("uuid"),
        securityAnswerOne = str("securityAnswerOne"),
        securityAnswerTwo = str("securityAnswerTwo"),
        securityAnswerThree = str("securityAnswerThree"),
        newPassword = str("newPassword"),
        confirmPassword = str("confirmPassword"),
    )
}
