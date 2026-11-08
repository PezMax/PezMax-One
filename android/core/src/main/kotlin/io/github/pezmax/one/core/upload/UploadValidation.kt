package io.github.pezmax.one.core.upload

import io.github.pezmax.one.core.auth.AuthValidation
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction

/**
 * 上传前的本地校验（spec/fixtures/upload/cases.json）。
 *
 * 两套规则不要混：资料上传收 14 种格式、doc/docx/ppt/pptx 转档；电子书只收 4 种、不转档。
 * 计数口径也不要统一：`fileName` / `ebookName` / `author` 按 rune，`fileSubject` / `fileSchool` /
 * `ebookSubject` 按 UTF-8 字节（一个汉字 3 字节，中文科目实际只有 21 个字）。
 */
object UploadValidation {

    const val HARD_CONVERT_LIMIT_MB = 200

    val FILE_EXTENSIONS = setOf(
        "doc", "docx", "ppt", "pptx", "pdf", "jpg", "jpeg", "png", "webp", "txt", "md", "epub", "mobi", "azw3",
    )
    val EBOOK_EXTENSIONS = setOf("pdf", "epub", "mobi", "azw3")
    val CONVERT_EXTENSIONS = setOf("doc", "docx", "ppt", "pptx")
    val TEXT_EXTENSIONS = setOf("txt", "md")
    /** 能进 ptmj_file 但不会出现在电子书列表里：界面要引导到电子书上传入口。 */
    val FILE_ONLY_EBOOK_EXTENSIONS = setOf("epub", "mobi", "azw3")

    const val ERR_FILE_FORMAT = "不支持的文件格式：仅支持 doc/docx/pdf/jpg/png/webp/txt/md/ppt/pptx"
    const val ERR_EBOOK_FORMAT = "不支持的电子书格式：仅支持 pdf/epub/mobi/azw3"
    const val ERR_SUBJECT = "科目不能为空且不超过 64 个字符"
    const val ERR_EBOOK_SUBJECT = "学科分类不能为空且不超过 64 个字符"
    const val ERR_SCHOOL = "学校名称不能超过 100 个字符"
    const val ERR_YEAR = "年份需在 1900 到 2100 之间"
    const val ERR_FILE_TYPE = "文件类型不正确"
    const val ERR_EBOOK_TYPE = "电子书类型不正确"
    const val ERR_FILE_NAME = "文件名不能为空且不超过 255 个字符"

    fun extensionOf(fileName: String): String {
        val dot = fileName.lastIndexOf('.')
        if (dot < 0 || dot == fileName.length - 1) return ""
        return fileName.substring(dot + 1).lowercase()
    }

    data class FileCheck(
        val accept: Boolean,
        val reason: String? = null,
        val convertsToPdf: Boolean = false,
        val needsUtf8Check: Boolean = false,
        val warnsNotEbook: Boolean = false,
    )

    fun checkFile(fileName: String, sizeBytes: Long, maxUploadMb: Int = 200): FileCheck {
        val ext = extensionOf(fileName)
        if (ext.isEmpty() || ext !in FILE_EXTENSIONS) {
            return FileCheck(accept = false, reason = ERR_FILE_FORMAT)
        }
        val converts = ext in CONVERT_EXTENSIONS
        val maxBytes = maxUploadMb.toLong() * 1024 * 1024
        val hardConvertBytes = HARD_CONVERT_LIMIT_MB.toLong() * 1024 * 1024
        if (converts && sizeBytes > hardConvertBytes) {
            return FileCheck(accept = false, reason = "转档文件不能超过 ${HARD_CONVERT_LIMIT_MB}MB")
        }
        // 0 字节与超限走同一条上游分支（`file.Size <= 0 || file.Size > maxBytes`）。
        if (sizeBytes <= 0 || sizeBytes > maxBytes) {
            return FileCheck(accept = false, reason = "文件过大：不能超过 ${maxUploadMb}MB")
        }
        return FileCheck(
            accept = true,
            convertsToPdf = converts,
            needsUtf8Check = ext in TEXT_EXTENSIONS,
            warnsNotEbook = ext in FILE_ONLY_EBOOK_EXTENSIONS,
        )
    }

    data class MetaCheck(
        val accept: Boolean,
        val reason: String? = null,
        val warnsNotEbook: Boolean = false,
    )

    fun validateFileMeta(
        fileName: String,
        fileSubject: String,
        fileSchool: String = "",
        fileYear: Int,
        fileType: Int,
    ): MetaCheck {
        if (AuthValidation.runeLength(fileName) !in 1..255) return MetaCheck(false, ERR_FILE_NAME)
        if (AuthValidation.trimSpace(fileSubject).isEmpty() || AuthValidation.utf8Length(fileSubject) > 64) {
            return MetaCheck(false, ERR_SUBJECT)
        }
        if (AuthValidation.utf8Length(fileSchool) > 100) return MetaCheck(false, ERR_SCHOOL)
        if (fileYear !in 1900..2100) return MetaCheck(false, ERR_YEAR)
        if (fileType !in 1..7) return MetaCheck(false, ERR_FILE_TYPE)
        return MetaCheck(accept = true, warnsNotEbook = fileType == 7)
    }

    fun validateEbookMeta(
        fileName: String,
        ebookName: String,
        ebookSubject: String,
        ebookType: Int,
        author: String = "",
        publisher: String = "",
    ): MetaCheck {
        if (extensionOf(fileName) !in EBOOK_EXTENSIONS) return MetaCheck(false, ERR_EBOOK_FORMAT)
        if (AuthValidation.runeLength(ebookName) !in 1..255) return MetaCheck(false, "书名不能为空且不超过 255 个字符")
        if (AuthValidation.trimSpace(ebookSubject).isEmpty() || AuthValidation.utf8Length(ebookSubject) > 64) {
            return MetaCheck(false, ERR_EBOOK_SUBJECT)
        }
        if (ebookType !in 1..4) return MetaCheck(false, ERR_EBOOK_TYPE)
        if (AuthValidation.runeLength(author) > 255) return MetaCheck(false, "作者不能超过 255 个字符")
        if (AuthValidation.runeLength(publisher) > 255) return MetaCheck(false, "出版社不能超过 255 个字符")
        return MetaCheck(accept = true)
    }

    data class BatchItem(val fileName: String, val sizeBytes: Long)

    data class BatchResult(val items: List<Pair<String, FileCheck>>)

    fun validateBatch(files: List<BatchItem>, maxUploadMb: Int = 200): BatchResult =
        BatchResult(files.map { it.fileName to checkFile(it.fileName, it.sizeBytes, maxUploadMb) })

    /**
     * UTF-8 判定：严格解码到末尾，截断的多字节序列在收尾时报错；UTF-8 BOM 跳过。
     * 空文件合法——这里只判「有没有坏字节」，体积是另一条规则。
     */
    fun isUtf8(bytes: ByteArray): Boolean {
        val start = if (bytes.size >= 3 &&
            bytes[0] == 0xEF.toByte() && bytes[1] == 0xBB.toByte() && bytes[2] == 0xBF.toByte()
        ) {
            3
        } else {
            0
        }
        val decoder = Charsets.UTF_8.newDecoder()
            .onMalformedInput(CodingErrorAction.REPORT)
            .onUnmappableCharacter(CodingErrorAction.REPORT)
        return try {
            decoder.decode(ByteBuffer.wrap(bytes, start, bytes.size - start))
            true
        } catch (_: Exception) {
            false
        }
    }
}
