#### 需要检索的文档格式如code_snippet中，以json格式输写
####在windows下cmd中运行时命令cat 改为type

```
type code_snippet.json | tantivy index -i ./code_snippet
```
#### 在windows下cmd中运行时，需要将cmd客户端编码的改为UTF-8，否则中文生成检索会乱码
1.临时修改cmd输入命令：
```
CHCP 65001
```
#### 注意：本人在powershell中用以上面方式改为UTF-8不启作用，请留意！！

#### 欢迎在code_snippet.json中添加常用的代码片段
例：{"moudle":"rust","title":"diesel","body":"migration generate xxxx","remark":"生成迁移"}
说明：
 moudle：模块 以语言分类，即python,java,rust,liunx,c,c++等
 title： 标题  当前代码段的标题
 body:   代码段内容
 remark: 备注说明