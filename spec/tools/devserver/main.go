// PezMax 本地开发服务器：按 `spec/api/endpoints.md` 的形状提供 /datum/* 的一份最小实现。
//
// 它**不是**后端，也不打算变成后端：
//   - 数据全在内存里，重启就清空；种子数据与 `PreviewData` 对齐，方便对着界面看。
//   - 会话只有一个固定 token（`-token`），**没有任何鉴权**，只监听 127.0.0.1。
//   - 通知 / 收藏 / 我的下载这些还没做界面验证的端点先回空列表，形状对、内容空。
//   - `/datum/download/file` 会现场生成一份 PDF（`pages` 参数控制页数），用来验 W2 的大文件。
//
// 契约的权威仍然是 spec：这里的形状若与 spec 不一致，改这里，不是改 spec。
package main

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

const (
	devUserID   = 7
	devUserName = "dev"
)

type File struct {
	ID          int64  `json:"id"`
	FileID      int64  `json:"fileId"`
	Label       string `json:"label"`
	FileName    string `json:"fileName"`
	FileURL     string `json:"fileUrl"`
	FileSize    int64  `json:"fileSize"`
	FileFormat  string `json:"fileFormat"`
	FileYear    int64  `json:"fileYear"`
	FileType    int64  `json:"fileType"`
	FileSchool  string `json:"fileSchool"`
	FileSubject string `json:"fileSubject"`
	FileStatus  int64  `json:"fileStatus"`
	Reviewer    string `json:"reviewer"`
	DelFlag     int64  `json:"delFlag"`
	UserID      int64  `json:"userId"`
	CreateBy    string `json:"createBy"`
	CreateTime  string `json:"createTime"`
	UpdateBy    string `json:"updateBy"`
	UpdateTime  string `json:"updateTime"`
	Remark      string `json:"remark"`
	Type        string `json:"type"`
}

type Ebook struct {
	ID           int64  `json:"id"`
	EbookID      int64  `json:"ebookId"`
	Label        string `json:"label"`
	EbookName    string `json:"ebookName"`
	Author       string `json:"author"`
	Publisher    string `json:"publisher"`
	CoverURL     string `json:"coverUrl"`
	EbookURL     string `json:"ebookUrl"`
	EbookSize    int64  `json:"ebookSize"`
	EbookFormat  string `json:"ebookFormat"`
	EbookSubject string `json:"ebookSubject"`
	EbookType    int64  `json:"ebookType"`
	EbookStatus  int64  `json:"ebookStatus"`
	UserID       int64  `json:"userId"`
	Remark       string `json:"remark"`
}

type store struct {
	mu     sync.Mutex
	files  []File
	ebooks []Ebook
	token  string
}

func main() {
	addr := flag.String("addr", "127.0.0.1:9033", "监听地址（只绑本机）")
	token := flag.String("token", "pezmax-dev-token", "会话 token；客户端登录后拿到它")
	upstream := flag.String("ref", "", "（可选）把未实现的路径转发到这个真后端，例如 http://127.0.0.1:8080")
	flag.Parse()

	s := &store{token: *token}
	s.seed()

	mux := http.NewServeMux()
	s.routes(mux)

	handler := http.Handler(mux)
	if *upstream != "" {
		handler = proxyFallback(*upstream, mux)
		log.Printf("未实现的路径转发到 %s", *upstream)
	}

	server := &http.Server{
		Addr:              *addr,
		Handler:           logRequests(handler),
		ReadHeaderTimeout: 10 * time.Second,
	}

	log.Printf("PezMax 本地开发服务器: http://%s", *addr)
	log.Printf("登录：任意用户名 + 任意密码 + 任意验证码（token=%s）", *token)
	if err := server.ListenAndServe(); err != nil {
		log.Fatal(err)
	}
}

// ---------------------------------------------------------------- 路由

func (s *store) routes(mux *http.ServeMux) {
	// 账户
	mux.HandleFunc("GET /datum/user/captchaImage", s.captcha)
	mux.HandleFunc("POST /datum/user/login", s.login)
	mux.HandleFunc("POST /datum/user/logout", s.logout)
	mux.HandleFunc("GET /datum/user/getInfo", s.guard(s.getInfo))
	mux.HandleFunc("GET /datum/user/securityQuestions", func(w http.ResponseWriter, r *http.Request) {
		ok(w, []string{"你的学号是？", "你的班主任姓？", "你的出生城市？"})
	})

	// 个人资料
	mux.HandleFunc("GET /datum/desktop/user/profile", s.guard(s.profile))
	mux.HandleFunc("GET /datum/desktop/user/profile/stats", s.guard(func(w http.ResponseWriter, r *http.Request) {
		ok(w, map[string]any{
			"uploadCount": 0, "downloadCount": 0, "favoriteCount": 0, "fileFavoriteCount": 0,
			"bookmarkFavoriteCount": 0,
		})
	}))

	// 资料
	mux.HandleFunc("GET /datum/file/tree", s.tree)
	mux.HandleFunc("GET /datum/file/list", s.listFiles)
	mux.HandleFunc("GET /datum/file/search", s.search)
	mux.HandleFunc("GET /datum/file/subjects", s.subjects)
	mux.HandleFunc("GET /datum/file/schools", s.schools)
	mux.HandleFunc("GET /datum/file/schools/check", func(w http.ResponseWriter, r *http.Request) {
		name := strings.TrimSpace(r.URL.Query().Get("schoolName"))
		if name == "" {
			fail(w, http.StatusBadRequest, "学校名称不能为空")
			return
		}
		ok(w, map[string]any{"exists": true})
	})
	mux.HandleFunc("GET /datum/file/content", s.fileContent)
	mux.HandleFunc("GET /datum/file/{fileId}", s.fileDetail)
	mux.HandleFunc("POST /datum/file", s.guard(s.upload))
	mux.HandleFunc("PUT /datum/file", s.guard(s.edit))
	mux.HandleFunc("DELETE /datum/file/{fileId}", s.guard(s.remove))

	// 电子书
	mux.HandleFunc("GET /datum/ebook/list", s.listEbooks)
	mux.HandleFunc("GET /datum/ebook/subjects", s.ebookSubjects)
	mux.HandleFunc("GET /datum/ebook/content", s.ebookContent)

	// 排行
	mux.HandleFunc("GET /datum/user/rank", func(w http.ResponseWriter, r *http.Request) {
		ok(w, []map[string]any{
			{"userId": 1, "userName": "林同学", "avatar": "", "count": 128, "uploads": 128},
			{"userId": 2, "userName": "陈同学", "avatar": "", "count": 96, "uploads": 96},
			{"userId": 3, "userName": "周同学", "avatar": "", "count": 74, "uploads": 74},
		})
	})

	// 下载（带 Range / ETag，和上游 ServeContent 的行为一致）
	mux.HandleFunc("GET /datum/download/file", s.downloadFile)
	mux.HandleFunc("GET /datum/download/ebook", s.downloadEbook)

	// 还没接界面验证的：形状对、内容空
	emptyList := func(w http.ResponseWriter, r *http.Request) { rows(w, []any{}, 0) }
	mux.HandleFunc("GET /datum/notification/list", s.guard(emptyList))
	mux.HandleFunc("GET /system/notification/user/popup", s.guard(func(w http.ResponseWriter, r *http.Request) { ok(w, []any{}) }))
	mux.HandleFunc("GET /datum/desktop/download/list/{userId}", s.guard(emptyList))
	mux.HandleFunc("GET /datum/desktop/favorite/list/{userId}", s.guard(emptyList))
	mux.HandleFunc("GET /datum/ebook/favorite/list", s.guard(emptyList))
	mux.HandleFunc("GET /datum/report/list", s.guard(emptyList))
	mux.HandleFunc("GET /datum/ebook/report/list", s.guard(emptyList))
	mux.HandleFunc("GET /datum/ebook/favorite", func(w http.ResponseWriter, r *http.Request) {
		ok(w, map[string]any{"favorited": false})
	})
	mux.HandleFunc("GET /datum/file/favorite", func(w http.ResponseWriter, r *http.Request) {
		ok(w, map[string]any{"favorited": false})
	})
}

// ---------------------------------------------------------------- 账户

func (s *store) captcha(w http.ResponseWriter, r *http.Request) {
	// captchaEnabled=false：本地调试不该被验证码挡住；客户端仍会让用户抄一个验证码，服务端忽略它。
	ok(w, map[string]any{"captchaEnabled": false, "uuid": "dev-uuid", "img": ""})
}

func (s *store) login(w http.ResponseWriter, r *http.Request) {
	var body map[string]any
	_ = json.NewDecoder(r.Body).Decode(&body)
	expires := time.Now().Add(120 * time.Hour).UnixMilli()
	ok(w, map[string]any{"token": s.token, "expiresAt": expires})
}

func (s *store) logout(w http.ResponseWriter, r *http.Request) { ok(w, true) }

func (s *store) getInfo(w http.ResponseWriter, r *http.Request) {
	ok(w, map[string]any{
		"user": map[string]any{
			"userId": devUserID, "userName": devUserName, "nickName": "本地调试",
			"avatar": "", "count": 3, "status": "1",
		},
		"roles": []string{"dev"}, "permissions": []string{},
	})
}

func (s *store) profile(w http.ResponseWriter, r *http.Request) {
	ok(w, map[string]any{
		"userId": devUserID, "userName": devUserName, "nickName": "本地调试",
		"avatar": "", "count": 3, "status": "1",
	})
}

// ---------------------------------------------------------------- 资料

func (s *store) tree(w http.ResponseWriter, r *http.Request) {
	const hash = "dev-tree-v1"
	if r.URL.Query().Get("hash") == hash {
		// 形状与上游一致：unchanged 时不带 data，客户端沿用本地缓存。
		writeJSON(w, http.StatusOK, map[string]any{
			"code": 200, "msg": "ok", "hash": hash, "unchanged": true, "data": nil,
		})
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	bySubject := map[string][]File{}
	for _, f := range s.files {
		if f.FileStatus != 1 || f.DelFlag != 0 {
			continue
		}
		bySubject[f.FileSubject] = append(bySubject[f.FileSubject], f)
	}

	subjects := make([]string, 0, len(bySubject))
	for name := range bySubject {
		subjects = append(subjects, name)
	}
	sort.Strings(subjects)

	nodes := make([]any, 0, len(subjects))
	for _, subject := range subjects {
		schools := map[string][]File{}
		for _, f := range bySubject[subject] {
			schools[f.FileSchool] = append(schools[f.FileSchool], f)
		}
		schoolNames := make([]string, 0, len(schools))
		for name := range schools {
			schoolNames = append(schoolNames, name)
		}
		sort.Strings(schoolNames)

		schoolNodes := make([]any, 0, len(schoolNames))
		for _, school := range schoolNames {
			years := map[int64][]File{}
			for _, f := range schools[school] {
				years[f.FileYear] = append(years[f.FileYear], f)
			}
			yearValues := make([]int64, 0, len(years))
			for year := range years {
				yearValues = append(yearValues, year)
			}
			sort.Slice(yearValues, func(i, j int) bool { return yearValues[i] > yearValues[j] })

			yearNodes := make([]any, 0, len(yearValues))
			for _, year := range yearValues {
				leaves := make([]any, 0, len(years[year]))
				for _, f := range years[year] {
					leaves = append(leaves, leaf(f))
				}
				yearNodes = append(yearNodes, map[string]any{
					"id":    "folder-" + school + "-" + strconv.FormatInt(year, 10),
					"label": strconv.FormatInt(year, 10), "type": "folder", "children": leaves,
				})
			}

			schoolNodes = append(schoolNodes, map[string]any{
				"id": "folder-" + school, "label": school, "type": "folder", "children": yearNodes,
			})
		}

		nodes = append(nodes, map[string]any{
			"id": "folder-" + subject, "label": subject, "type": "folder", "children": schoolNodes,
		})
	}

	writeJSON(w, http.StatusOK, map[string]any{
		"code": 200, "msg": "ok", "hash": hash, "unchanged": false, "data": nodes,
	})
}

func leaf(f File) map[string]any {
	node := map[string]any{
		"id": "file-" + strconv.FormatInt(f.FileID, 10), "fileId": f.FileID, "label": f.FileName,
		"type": "file", "children": []any{}, "fileUrl": f.FileURL, "fileFormat": f.FileFormat,
	}
	for key, value := range fileMap(f) {
		node[key] = value
	}
	return node
}

func (s *store) listFiles(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()
	wantUser := query.Get("userId")
	if wantUser != "" && wantUser != strconv.FormatInt(devUserID, 10) {
		// 上游：userId 必须等于会话用户，否则 403。
		fail(w, http.StatusForbidden, "只能查看自己的上传")
		return
	}

	s.mu.Lock()
	rows := make([]any, 0, len(s.files))
	for _, f := range s.files {
		if f.DelFlag != 0 {
			continue
		}
		if wantUser == "" {
			if f.FileStatus != 1 {
				continue
			}
		} else if f.UserID != devUserID {
			continue
		}
		if v := query.Get("fileSubject"); v != "" && f.FileSubject != v {
			continue
		}
		if v := query.Get("fileSchool"); v != "" && f.FileSchool != v {
			continue
		}
		if v := query.Get("fileType"); v != "" && strconv.FormatInt(f.FileType, 10) != v {
			continue
		}
		if v := query.Get("fileYear"); v != "" && strconv.FormatInt(f.FileYear, 10) != v {
			continue
		}
		rows = append(rows, fileMap(f))
	}
	s.mu.Unlock()

	sort.Slice(rows, func(i, j int) bool {
		return rows[i].(map[string]any)["fileId"].(int64) > rows[j].(map[string]any)["fileId"].(int64)
	})

	pageNum, pageSize := pagination(query)
	rows = paginate(rows, pageNum, pageSize)
	writeJSON(w, http.StatusOK, map[string]any{
		"code": 200, "msg": "ok", "rows": rows, "total": len(s.files),
	})
}

func (s *store) search(w http.ResponseWriter, r *http.Request) {
	keyword := strings.TrimSpace(r.URL.Query().Get("keyword"))
	if keyword == "" {
		ok(w, []any{})
		return
	}

	s.mu.Lock()
	found := make([]any, 0, 8)
	for _, f := range s.files {
		if f.FileStatus != 1 || f.DelFlag != 0 {
			continue
		}
		if strings.Contains(f.FileName, keyword) || strings.Contains(f.FileSubject, keyword) ||
			strings.Contains(f.FileSchool, keyword) {
			found = append(found, fileMap(f))
		}
	}
	s.mu.Unlock()

	if len(found) > 50 {
		found = found[:50]
	}
	ok(w, found)
}

func (s *store) subjects(w http.ResponseWriter, r *http.Request) {
	s.mu.Lock()
	counts := map[string]int{}
	for _, f := range s.files {
		if f.FileStatus == 1 && f.DelFlag == 0 {
			counts[f.FileSubject]++
		}
	}
	s.mu.Unlock()

	names := make([]string, 0, len(counts))
	for name := range counts {
		names = append(names, name)
	}
	sort.Strings(names)

	// 上游这个结构体没有 JSON tag，字段是首字母大写的 Subject / Total。
	items := make([]any, 0, len(names))
	for _, name := range names {
		items = append(items, map[string]any{"Subject": name, "Total": counts[name]})
	}
	ok(w, items)
}

func (s *store) schools(w http.ResponseWriter, r *http.Request) {
	keyword := strings.TrimSpace(r.URL.Query().Get("keyword"))

	s.mu.Lock()
	set := map[string]struct{}{}
	for _, f := range s.files {
		if f.FileStatus != 1 || f.DelFlag != 0 || f.FileSchool == "" {
			continue
		}
		if keyword != "" && !strings.Contains(f.FileSchool, keyword) {
			continue
		}
		set[f.FileSchool] = struct{}{}
	}
	s.mu.Unlock()

	names := make([]string, 0, len(set))
	for name := range set {
		names = append(names, name)
	}
	sort.Strings(names)
	ok(w, names)
}

func (s *store) fileDetail(w http.ResponseWriter, r *http.Request) {
	id, err := strconv.ParseInt(r.PathValue("fileId"), 10, 64)
	if err != nil {
		fail(w, http.StatusBadRequest, "文件 ID 不正确")
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()
	for _, f := range s.files {
		if f.FileID != id || f.DelFlag != 0 {
			continue
		}
		// 待审件只有属主可见，否则 404（与上游一致）。
		if f.FileStatus != 1 && f.UserID != devUserID {
			fail(w, http.StatusNotFound, "文件不存在")
			return
		}
		ok(w, fileMap(f))
		return
	}
	fail(w, http.StatusNotFound, "文件不存在")
}

func (s *store) fileContent(w http.ResponseWriter, r *http.Request) {
	id, _ := strconv.ParseInt(r.URL.Query().Get("fileId"), 10, 64)
	s.mu.Lock()
	var found *File
	for i := range s.files {
		if s.files[i].FileID == id {
			found = &s.files[i]
			break
		}
	}
	s.mu.Unlock()

	if found == nil {
		fail(w, http.StatusNotFound, "文件不存在")
		return
	}

	format := strings.ToLower(found.FileFormat)
	if format != "txt" && format != "md" {
		fail(w, http.StatusBadRequest, "仅文本文件支持内容预览")
		return
	}

	w.Header().Set("Content-Type", "text/plain; charset=utf-8")
	fmt.Fprintf(w, "# %s\n\n本地开发服务器生成的文本内容。\n\n- 科目：%s\n- 学校：%s\n- 年份：%d\n",
		found.FileName, found.FileSubject, found.FileSchool, found.FileYear)
}

func (s *store) upload(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseMultipartForm(64 << 20); err != nil {
		fail(w, http.StatusBadRequest, "上传体解析失败")
		return
	}

	name := r.FormValue("fileName")
	subject := r.FormValue("fileSubject")
	school := r.FormValue("fileSchool")
	year, _ := strconv.ParseInt(r.FormValue("fileYear"), 10, 64)
	fileType, _ := strconv.ParseInt(r.FormValue("fileType"), 10, 64)
	remark := r.FormValue("remark")

	if strings.TrimSpace(name) == "" || strings.TrimSpace(subject) == "" {
		fail(w, http.StatusBadRequest, "文件名与科目不能为空")
		return
	}

	var size int64
	if file, _, err := r.FormFile("file"); err == nil {
		size, _ = io.Copy(io.Discard, file)
		_ = file.Close()
	}

	s.mu.Lock()
	id := int64(1)
	for _, f := range s.files {
		if f.FileID >= id {
			id = f.FileID + 1
		}
	}
	now := time.Now().Format("2006-01-02 15:04:05")
	format := strings.TrimPrefix(strings.ToLower(filepath.Ext(name)), ".")
	record := File{
		ID: id, FileID: id, Label: name, FileName: name,
		FileURL: "http://" + "local-dev/未审核/" + name, FileSize: size, FileFormat: format,
		FileYear: year, FileType: fileType, FileSchool: school, FileSubject: subject,
		FileStatus: 0, UserID: devUserID, CreateBy: devUserName, CreateTime: now,
		UpdateBy: devUserName, UpdateTime: now, Remark: remark,
	}
	s.files = append(s.files, record)
	s.mu.Unlock()

	ok(w, map[string]any{"fileId": id, "fileUrl": record.FileURL, "fileName": name})
}

func (s *store) edit(w http.ResponseWriter, r *http.Request) {
	var body struct {
		FileID      int64  `json:"fileId"`
		FileName    string `json:"fileName"`
		FileYear    int64  `json:"fileYear"`
		FileType    int64  `json:"fileType"`
		FileSchool  string `json:"fileSchool"`
		FileSubject string `json:"fileSubject"`
		Remark      string `json:"remark"`
	}
	if err := json.NewDecoder(r.Body).Decode(&body); err != nil || body.FileID <= 0 {
		fail(w, http.StatusBadRequest, "文件 ID 不能为空")
		return
	}

	s.mu.Lock()
	defer s.mu.Unlock()
	for i := range s.files {
		f := &s.files[i]
		if f.FileID != body.FileID || f.UserID != devUserID || f.DelFlag != 0 {
			continue
		}
		// 整组覆盖：和上游一样的 SET 列表（remark 不在请求里就等于清空）。
		f.FileName, f.Label = body.FileName, body.FileName
		f.FileYear, f.FileType, f.FileSchool, f.FileSubject, f.Remark = body.FileYear, body.FileType, body.FileSchool, body.FileSubject, body.Remark
		f.UpdateBy = devUserName
		f.UpdateTime = time.Now().Format("2006-01-02 15:04:05")
		ok(w, true)
		return
	}
	fail(w, http.StatusNotFound, "文件不存在或无权修改")
}

func (s *store) remove(w http.ResponseWriter, r *http.Request) {
	id, _ := strconv.ParseInt(r.PathValue("fileId"), 10, 64)

	s.mu.Lock()
	defer s.mu.Unlock()
	for i := range s.files {
		f := &s.files[i]
		if f.FileID != id || f.UserID != devUserID || f.DelFlag != 0 {
			continue
		}
		f.DelFlag = 1
		f.UpdateTime = time.Now().Format("2006-01-02 15:04:05")
		ok(w, true)
		return
	}
	fail(w, http.StatusNotFound, "文件不存在或无权删除")
}

// ---------------------------------------------------------------- 电子书

func (s *store) listEbooks(w http.ResponseWriter, r *http.Request) {
	s.mu.Lock()
	rows := make([]any, 0, len(s.ebooks))
	for _, e := range s.ebooks {
		if e.EbookStatus == 1 {
			rows = append(rows, ebookMap(e))
		}
	}
	s.mu.Unlock()

	pageNum, pageSize := pagination(r.URL.Query())
	paged := paginate(rows, pageNum, pageSize)
	writeJSON(w, http.StatusOK, map[string]any{"code": 200, "msg": "ok", "rows": paged, "total": len(rows)})
}

func (s *store) ebookSubjects(w http.ResponseWriter, r *http.Request) {
	s.mu.Lock()
	counts := map[string]int{}
	for _, e := range s.ebooks {
		counts[e.EbookSubject]++
	}
	s.mu.Unlock()

	items := make([]any, 0, len(counts))
	for name, total := range counts {
		items = append(items, map[string]any{"Subject": name, "Total": total})
	}
	ok(w, items)
}

func (s *store) ebookContent(w http.ResponseWriter, r *http.Request) {
	id, _ := strconv.ParseInt(r.URL.Query().Get("ebookId"), 10, 64)

	s.mu.Lock()
	var found *Ebook
	for i := range s.ebooks {
		if s.ebooks[i].EbookID == id {
			found = &s.ebooks[i]
			break
		}
	}
	s.mu.Unlock()

	if found == nil {
		fail(w, http.StatusNotFound, "电子书不存在")
		return
	}

	if strings.EqualFold(found.EbookFormat, "epub") {
		writeBytes(w, r, buildEpub(found.EbookName), found.EbookName+".epub", "application/epub+zip",
			fmt.Sprintf(`"ebook-%d-%d"`, found.EbookID, len(buildEpub(found.EbookName))))
		return
	}

	pdf := buildPDF(found.EbookName, 6)
	writeBytes(w, r, pdf, found.EbookName+".pdf", "application/pdf",
		fmt.Sprintf(`"ebook-%d-%d"`, found.EbookID, len(pdf)))
}

// ---------------------------------------------------------------- 下载

func (s *store) downloadFile(w http.ResponseWriter, r *http.Request) {
	id, _ := strconv.ParseInt(r.URL.Query().Get("fileId"), 10, 64)
	pages := 4
	if v := r.URL.Query().Get("pages"); v != "" {
		if parsed, err := strconv.Atoi(v); err == nil && parsed > 0 && parsed <= 500 {
			pages = parsed
		}
	}

	s.mu.Lock()
	var found *File
	for i := range s.files {
		if s.files[i].FileID == id {
			found = &s.files[i]
			break
		}
	}
	s.mu.Unlock()

	if found == nil {
		fail(w, http.StatusNotFound, "文件不存在或已删除")
		return
	}
	// 上游：非已通过且非属主 → 404。
	if found.FileStatus != 1 && found.UserID != devUserID {
		fail(w, http.StatusNotFound, "文件不存在或已删除")
		return
	}

	if strings.EqualFold(found.FileFormat, "txt") || strings.EqualFold(found.FileFormat, "md") {
		body := []byte("# " + found.FileName + "\n\n本地开发服务器的文本内容。\n")
		writeBytes(w, r, body, found.FileName, "text/plain; charset=utf-8",
			fmt.Sprintf(`"datum-%d-%d"`, found.FileID, len(body)))
		return
	}

	pdf := buildPDF(found.FileName, pages)
	writeBytes(w, r, pdf, found.FileName, "application/pdf",
		fmt.Sprintf(`"datum-%d-%d"`, found.FileID, len(pdf)))
}

func (s *store) downloadEbook(w http.ResponseWriter, r *http.Request) {
	id, _ := strconv.ParseInt(r.URL.Query().Get("ebookId"), 10, 64)

	s.mu.Lock()
	var found *Ebook
	for i := range s.ebooks {
		if s.ebooks[i].EbookID == id {
			found = &s.ebooks[i]
			break
		}
	}
	s.mu.Unlock()

	if found == nil {
		fail(w, http.StatusNotFound, "电子书不存在")
		return
	}

	if strings.EqualFold(found.EbookFormat, "epub") {
		body := buildEpub(found.EbookName)
		writeBytes(w, r, body, found.EbookName+".epub", "application/epub+zip",
			fmt.Sprintf(`"ebook-%d-%d"`, found.EbookID, len(body)))
		return
	}

	body := buildPDF(found.EbookName, 6)
	writeBytes(w, r, body, found.EbookName+".pdf", "application/pdf",
		fmt.Sprintf(`"ebook-%d-%d"`, found.EbookID, len(body)))
}

// ---------------------------------------------------------------- 种子数据

func (s *store) seed() {
	now := time.Now().Format("2006-01-02 15:04:05")
	add := func(id int64, name, subject, school string, year, fileType int64, format string, size, status int64) {
		s.files = append(s.files, File{
			ID: id, FileID: id, Label: name, FileName: name,
			FileURL:  "http://local-dev/datum/download/file?fileId=" + strconv.FormatInt(id, 10),
			FileSize: size, FileFormat: format, FileYear: year, FileType: fileType,
			FileSchool: school, FileSubject: subject, FileStatus: status,
			UserID: devUserID, CreateBy: devUserName, CreateTime: now,
			UpdateBy: devUserName, UpdateTime: now, Type: "file",
		})
	}

	add(101, "高等数学期末试卷.pdf", "数学", "实验中学", 2025, 1, "pdf", 2457600, 1)
	add(102, "大学物理期中试题.pdf", "物理", "第一中学", 2025, 2, "pdf", 1835008, 1)
	add(103, "英语四级真题合集.pdf", "英语", "外国语学校", 2024, 4, "pdf", 524288, 1)
	add(104, "数据结构复习提纲.md", "计算机", "实验中学", 2025, 4, "md", 20480, 1)
	add(105, "化学实验报告模板.txt", "化学", "第二中学", 2024, 6, "txt", 8192, 1)
	add(106, "线性代数模拟卷.pdf", "数学", "实验中学", 2024, 2, "pdf", 3145728, 0)

	s.ebooks = append(s.ebooks,
		Ebook{
			ID: 201, EbookID: 201, Label: "高等数学（上册）", EbookName: "高等数学（上册）",
			Author: "同济大学数学系", Publisher: "高等教育出版社",
			EbookURL: "http://local-dev/datum/download/ebook?ebookId=201", EbookSize: 1048576,
			EbookFormat: "pdf", EbookSubject: "数学", EbookType: 1, EbookStatus: 1, UserID: devUserID,
		},
		Ebook{
			ID: 202, EbookID: 202, Label: "线性代数讲义", EbookName: "线性代数讲义",
			Author: "本地调试", Publisher: "内部讲义",
			EbookURL: "http://local-dev/datum/download/ebook?ebookId=202", EbookSize: 524288,
			EbookFormat: "epub", EbookSubject: "数学", EbookType: 2, EbookStatus: 1, UserID: devUserID,
		},
		Ebook{
			ID: 203, EbookID: 203, Label: "大学英语泛读", EbookName: "大学英语泛读",
			Author: "本地调试", Publisher: "外语教学与研究出版社",
			EbookURL: "http://local-dev/datum/download/ebook?ebookId=203", EbookSize: 786432,
			EbookFormat: "epub", EbookSubject: "英语", EbookType: 1, EbookStatus: 1, UserID: devUserID,
		},
	)
}

// ---------------------------------------------------------------- 生成 PDF / EPUB

// buildPDF 手写一个最小合法 PDF：页数可控，用来验「100 页的试卷」这条 W2。
func buildPDF(title string, pages int) []byte {
	if pages < 1 {
		pages = 1
	}

	var buffer bytes.Buffer
	offsets := make([]int, 0, pages+4)
	write := func(text string) {
		offsets = append(offsets, buffer.Len())
		buffer.WriteString(text)
	}

	buffer.WriteString("%PDF-1.4\n")
	write("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n")

	kids := make([]string, 0, pages)
	for i := 0; i < pages; i++ {
		kids = append(kids, fmt.Sprintf("%d 0 R", 4+i*2))
	}
	write(fmt.Sprintf("2 0 obj\n<< /Type /Pages /Kids [%s] /Count %d >>\nendobj\n",
		strings.Join(kids, " "), pages))
	write("3 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n")

	for i := 0; i < pages; i++ {
		pageObj := 4 + i*2
		contentObj := pageObj + 1
		write(fmt.Sprintf(
			"%d 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents %d 0 R >>\nendobj\n",
			pageObj, contentObj))

		content := fmt.Sprintf("BT /F1 22 Tf 72 780 Td (%s) Tj ET\nBT /F1 14 Tf 72 740 Td (Page %d / %d) Tj ET\n",
			escapePDF(title), i+1, pages)
		write(fmt.Sprintf("%d 0 obj\n<< /Length %d >>\nstream\n%sendstream\nendobj\n",
			contentObj, len(content), content))
	}

	xrefOffset := buffer.Len()
	buffer.WriteString(fmt.Sprintf("xref\n0 %d\n0000000000 65535 f \n", len(offsets)+1))
	for _, offset := range offsets {
		buffer.WriteString(fmt.Sprintf("%010d 00000 n \n", offset))
	}
	buffer.WriteString(fmt.Sprintf(
		"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n", len(offsets)+1, xrefOffset))

	return buffer.Bytes()
}

func escapePDF(text string) string {
	replacer := strings.NewReplacer("\\", "\\\\", "(", "\\(", ")", "\\)")
	// Helvetica 只有 ASCII 字形：非 ASCII 先换成问号，免得生成一堆乱码。
	var ascii strings.Builder
	for _, r := range text {
		if r < 128 {
			ascii.WriteRune(r)
		} else {
			ascii.WriteByte('?')
		}
	}
	return replacer.Replace(ascii.String())
}

// buildEpub 手写一个最小合法 EPUB（三章 + 目录），用来验 W3 的解析与虚拟主机。
func buildEpub(title string) []byte {
	var buffer bytes.Buffer
	writer := zip.NewWriter(&buffer)
	add := func(name string, body []byte, store bool) {
		header := &zip.FileHeader{Name: name, Modified: time.Now()}
		if store {
			header.Method = zip.Store
		} else {
			header.Method = zip.Deflate
		}
		entry, err := writer.CreateHeader(header)
		if err != nil {
			return
		}
		_, _ = entry.Write(body)
	}

	add("mimetype", []byte("application/epub+zip"), true)

	add("META-INF/container.xml", []byte(`<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>`), true)

	add("OEBPS/content.opf", []byte(fmt.Sprintf(`<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>%s</dc:title><dc:creator>本地调试</dc:creator>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
    <item id="c3" href="ch3.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="c1"/><itemref idref="c2"/><itemref idref="c3"/></spine>
</package>`, title)), true)

	add("OEBPS/nav.xhtml", []byte(`<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <body><nav epub:type="toc"><ol>
    <li><a href="ch1.xhtml">第一章 本地调试</a></li>
    <li><a href="ch2.xhtml">第二章 虚拟主机</a></li>
    <li><a href="ch3.xhtml">第三章 排版检查</a></li>
  </ol></nav></body>
</html>`), true)

	chapters := []string{"第一章 本地调试", "第二章 虚拟主机", "第三章 排版检查"}
	for i, name := range chapters {
		add(fmt.Sprintf("OEBPS/ch%d.xhtml", i+1), []byte(fmt.Sprintf(`<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>%s</title></head>
<body><h1>%s</h1>
<p>这一章由本地开发服务器生成，用来检查 WebView2 的虚拟主机映射、离线读取与中文排版。</p>
<p>段落里混合中英文：The quick brown fox 跳过懒狗。数字与标点：0123456789，。、；：！？</p>
<p>长段落用来触发换行：%s</p>
</body></html>`, name, name, strings.Repeat("排版检查用的长句。", 12))), true)
	}

	_ = writer.Close()
	return buffer.Bytes()
}

// ---------------------------------------------------------------- HTTP 工具

func ok(w http.ResponseWriter, data any) {
	writeJSON(w, http.StatusOK, map[string]any{"code": 200, "msg": "ok", "data": data})
}

func rows(w http.ResponseWriter, items []any, total int) {
	writeJSON(w, http.StatusOK, map[string]any{"code": 200, "msg": "ok", "rows": items, "total": total})
}

func fail(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, map[string]any{"code": status, "msg": message})
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func writeBytes(w http.ResponseWriter, r *http.Request, body []byte, name, contentType, etag string) {
	w.Header().Set("Content-Type", contentType)
	w.Header().Set("ETag", etag)
	// Accept-Encoding: identity 是客户端的要求；这里也不压缩。
	http.ServeContent(w, r, name, time.Time{}, bytes.NewReader(body))
}

func (s *store) guard(handler http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		header := r.Header.Get("Authorization")
		if !strings.HasPrefix(header, "Bearer ") || strings.TrimSpace(strings.TrimPrefix(header, "Bearer ")) != s.token {
			// 客户端靠 401 判断会话失效：这里必须回 401，不是 200 + code。
			writeJSON(w, http.StatusUnauthorized, map[string]any{"code": 401, "msg": "登录已失效，请重新登录"})
			return
		}
		handler(w, r)
	}
}

func pagination(query map[string][]string) (int, int) {
	pageNum, pageSize := 1, 20
	if v := first(query, "pageNum"); v != "" {
		if parsed, err := strconv.Atoi(v); err == nil && parsed > 0 {
			pageNum = parsed
		}
	}
	if v := first(query, "pageSize"); v != "" {
		if parsed, err := strconv.Atoi(v); err == nil && parsed > 0 {
			pageSize = parsed
		}
	}
	if pageSize > 100 {
		pageSize = 100
	}
	return pageNum, pageSize
}

func first(query map[string][]string, key string) string {
	if values, ok := query[key]; ok && len(values) > 0 {
		return values[0]
	}
	return ""
}

func paginate(items []any, pageNum, pageSize int) []any {
	start := (pageNum - 1) * pageSize
	if start >= len(items) {
		return []any{}
	}
	end := start + pageSize
	if end > len(items) {
		end = len(items)
	}
	return items[start:end]
}

func fileMap(f File) map[string]any {
	return map[string]any{
		"id": f.ID, "fileId": f.FileID, "label": f.Label, "fileName": f.FileName,
		"fileUrl": f.FileURL, "fileSize": f.FileSize, "fileFormat": f.FileFormat,
		"fileYear": f.FileYear, "fileType": f.FileType, "fileSchool": f.FileSchool,
		"fileSubject": f.FileSubject, "fileStatus": f.FileStatus, "reviewer": f.Reviewer,
		"delFlag": f.DelFlag, "userId": f.UserID, "createBy": f.CreateBy, "createTime": f.CreateTime,
		"updateBy": f.UpdateBy, "updateTime": f.UpdateTime, "remark": f.Remark, "type": "file",
	}
}

func ebookMap(e Ebook) map[string]any {
	return map[string]any{
		"id": e.ID, "ebookId": e.EbookID, "label": e.Label, "ebookName": e.EbookName,
		"author": e.Author, "publisher": e.Publisher, "coverUrl": e.CoverURL,
		"ebookUrl": e.EbookURL, "ebookSize": e.EbookSize, "ebookFormat": e.EbookFormat,
		"ebookSubject": e.EbookSubject, "ebookType": e.EbookType, "ebookStatus": e.EbookStatus,
		"userId": e.UserID, "remark": e.Remark,
	}
}

func logRequests(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		started := time.Now()
		recorder := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
		next.ServeHTTP(recorder, r)
		log.Printf("%-6s %-42s %d %s", r.Method, r.URL.RequestURI(), recorder.status, time.Since(started).Round(time.Millisecond))
	})
}

type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (s *statusRecorder) WriteHeader(status int) {
	s.status = status
	s.ResponseWriter.WriteHeader(status)
}

// proxyFallback：未注册的路径转给真后端，方便「本地 devserver + 真后端」混着调。
func proxyFallback(upstream string, local *http.ServeMux) http.Handler {
	upstream = strings.TrimSuffix(upstream, "/")
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		matched, _ := local.Handler(r)
		if matched != nil {
			local.ServeHTTP(w, r)
			return
		}

		request, err := http.NewRequestWithContext(r.Context(), r.Method, upstream+r.URL.RequestURI(), r.Body)
		if err != nil {
			fail(w, http.StatusBadGateway, "转发失败")
			return
		}
		request.Header = r.Header.Clone()
		response, err := http.DefaultClient.Do(request)
		if err != nil {
			fail(w, http.StatusBadGateway, "真后端连不上："+err.Error())
			return
		}
		defer response.Body.Close()
		for key, values := range response.Header {
			for _, value := range values {
				w.Header().Add(key, value)
			}
		}
		w.WriteHeader(response.StatusCode)
		_, _ = io.Copy(w, response.Body)
	})
}

func init() {
	log.SetFlags(log.Ltime)
	log.SetOutput(os.Stdout)
}
