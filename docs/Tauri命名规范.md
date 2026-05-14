Tauri命名规范
一、核心目标
在 Rust（后端） ↔ 前端（TypeScript/JS） 之间：
• 保持 类型一致 
• 自动完成 命名转换 
• 避免手动映射错误 
---
二、命名规则总览
类型	Rust	前端
变量 / 字段	snake_case	camelCase
类型（Struct / Enum）	PascalCase	PascalCase
Enum 变体	PascalCase	PascalCase
👉 核心：字段不同，类型相同
---
三、自动转换（关键机制）
使用 serde 自动处理命名差异：
#[serde(rename_all = "camelCase")]
作用：
Rust: user_name  →  前端: userName
---
四、标准写法
1️⃣ Struct（数据结构）
Rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Example {
    pub user_name: String,
    pub connection_id: String,
}
前端
export interface Example {
    userName: string;
    connectionId: string;
}
---
2️⃣ Enum（推荐写法）
Rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthMethod {
    Password { password: String },
    PublicKey { privateKeyPath: String },
}
前端（联合类型）
export type AuthMethod =
  | { Password: { password: string } }
  | { PublicKey: { privateKeyPath: string } };
---
五、重要规则（必须记住）
✅ 1. 类型名必须一致
Rust: SessionConfig
TS:   SessionConfig
---
✅ 2. 字段靠 serde 自动转换
不要手动改前端字段名
---
✅ 3. 缩写统一处理
正确	错误
connectionId	connectionID ❌
userId	userID ❌
---
✅ 4. 特殊字段手动指定
#[serde(rename = "id")]
pub session_id: String;
---
六、常见错误
❌ 忘记 serde
pub user_name: String   // → 前端收不到 userName
---
❌ 前端写 snake_case
user_name: string  // ❌
---
❌ Enum 不一致
Password   // Rust
password   // ❌ 不匹配
---
七、推荐开发流程
1. Rust 定义 Struct / Enum 
2. 加 #[serde(rename_all = "camelCase")] 
3. 前端按 camelCase 写 interface 
4. 测试 JSON 是否一致 
---
八、一句话总结
👉 Rust 写 snake_case + serde → 前端自动 camelCase
👉 类型名永远保持一致
