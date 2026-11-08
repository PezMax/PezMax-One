package io.github.pezmax.one.data

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * 凭据存储：Android Keystore 生成 AES-GCM 密钥，密文写应用私有文件。
 * 不用 `androidx.security:security-crypto`（已弃用，android/AGENTS.md 平台集成表）。
 *
 * 读失败（密钥被清、文件损坏）一律当「未登录」：宁可让用户重登，
 * 也不要在启动路径上抛异常。
 */
class CredentialStore(context: Context) {
    private val file = File(context.filesDir, "credentials.bin")
    private val keyStore: KeyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

    private var cached: Pair<String, Long>? = read()

    val token: String? get() = cached?.first

    val userId: Long get() = cached?.second ?: 0L

    fun save(token: String, userId: Long) {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, secretKey())
        val payload = "${userId}\n$token".toByteArray(Charsets.UTF_8)
        val encrypted = cipher.iv + cipher.doFinal(payload)
        runCatching { file.writeBytes(encrypted) }
        cached = token to userId
    }

    fun clear() {
        runCatching { file.delete() }
        cached = null
    }

    fun isLoggedIn(): Boolean = !token.isNullOrBlank()

    private fun read(): Pair<String, Long>? {
        if (!file.isFile) return null
        val bytes = runCatching { file.readBytes() }.getOrNull() ?: return null
        if (bytes.size <= IV_SIZE) return null
        return runCatching {
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, secretKey(), GCMParameterSpec(TAG_BITS, bytes, 0, IV_SIZE))
            val plain = String(cipher.doFinal(bytes, IV_SIZE, bytes.size - IV_SIZE), Charsets.UTF_8)
            val split = plain.split('\n', limit = 2)
            if (split.size != 2) return@runCatching null
            split[1] to (split[0].toLongOrNull() ?: 0L)
        }.getOrNull()
    }

    private fun secretKey(): SecretKey {
        (keyStore.getEntry(ALIAS, null) as? KeyStore.SecretKeyEntry)?.let { return it.secretKey }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true)
                .build(),
        )
        return generator.generateKey()
    }

    private companion object {
        const val ANDROID_KEYSTORE = "AndroidKeyStore"
        const val ALIAS = "pezmax.credentials"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val IV_SIZE = 12
        const val TAG_BITS = 128
    }
}

/** 验证码头只在展示时拼前缀；服务端不回 `data:` 前缀（auth.md §1）。 */
fun base64Image(base64: String): String = "data:image/jpeg;base64,$base64"
