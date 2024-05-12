use serde::Deserialize;

use serde_json::Value;
use serde_repr::Deserialize_repr;

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct Login {
    /// 用户对象
    pub user: Option<User>,
    /// 平台账号
    pub self_id: Option<String>,
    /// 平台名称
    pub platform: Option<String>,
    /// 登录状态
    pub status: Status,
}
#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    /// 用户 ID
    pub id: String,
    /// 用户名称
    pub name: Option<String>,
    /// 用户昵称
    pub nick: Option<String>,
    /// 用户头像
    pub avatar: Option<String>,
    /// 是否为机器人
    pub is_bot: Option<bool>,
}

#[derive(Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum Status {
    /// 离线
    OFFLINE = 0,
    /// 在线
    ONLINE = 1,
    /// 连接中
    CONNECT = 2,
    /// 断开连接
    DISCONNECT = 3,
    /// 重新连接
    RECONNECT = 4,
}

#[derive(Deserialize, Default, Clone)]
pub struct Event {
    /// 事件 ID
    pub id: i32,
    /// 事件类型
    #[serde(rename = "type")]
    pub _type: String,
    /// 接收者的平台名称
    pub platform: String,
    /// 接收者的平台账号
    pub self_id: String,
    /// 事件的时间戳
    pub timestamp: i64,
    /// 交互指令
    pub argv: Option<Argv>,
    /// 交互按钮
    pub button: Option<Button>,
    /// 事件所属的频道
    pub channel: Option<Channel>,
    /// 事件所属的群组
    pub guild: Option<Guild>,
    /// 事件的登录信息
    pub login: Option<Login>,
    /// 事件的目标成员
    pub member: Option<GuildMember>,
    /// 事件的消息
    pub message: Option<Message>,
    /// 事件的操作者
    pub operator: Option<User>,
    /// 事件的目标角色
    pub role: Option<GuildRole>,
    /// 事件的目标用户
    pub user: Option<User>,
}

#[derive(Deserialize, Clone)]
pub struct Argv {
    /// 指令名称
    pub name: String,
    /// 参数
    pub arguments: Vec<Value>,
    /// 选项
    pub options: Value,
}

#[derive(Deserialize, Clone)]
pub struct Button {
    /// 按钮 ID
    pub id: String,
}

#[derive(Deserialize, Clone)]
pub struct Channel {
    /// 频道 ID
    pub id: String,
    /// 频道类型
    #[serde(rename = "type")]
    _type: ChannelType,
    /// 频道名称
    pub name: Option<String>,
    /// 父频道 ID
    pub parent_id: Option<String>,
}
#[derive(Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum ChannelType {
    /// 文本频道
    TEXT = 0,
    /// 私聊频道
    DIRECT = 1,
    /// 分类频道
    CATEGORY = 2,
    /// 语音频道
    VOICE = 3,
}
#[derive(Deserialize, Clone)]
pub struct Guild {
    /// 群组 ID
    pub id: String,
    /// 群组名称
    pub name: Option<String>,
    /// 群组头像
    pub avatar: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct GuildMember {
    /// 用户对象
    pub user: Option<User>,
    /// 用户在群组中的名称
    pub nick: Option<String>,
    /// 用户在群组中的头像
    pub avatar: Option<String>,
    /// 加入时间
    pub joined_at: Option<i64>,
}

#[derive(Deserialize, Clone)]
pub struct Message {
    /// 消息 ID
    pub id: String,
    /// 消息内容
    pub content: String,
    /// 频道对象
    pub channel: Option<Channel>,
    /// 群组对象
    pub guild: Option<Guild>,
    /// 成员对象
    pub member: Option<GuildMember>,
    /// 用户对象
    pub user: Option<User>,
    /// 消息发送的时间戳
    pub created_at: Option<i64>,
    /// 消息修改的时间戳
    pub updated_at: Option<i64>,
}

#[derive(Deserialize, Clone)]
pub struct GuildRole {
    /// 角色 ID
    pub id: String,
    /// 角色名称
    pub name: Option<String>,
}

impl TryInto<kritor::event::EventStructure> for Event {
    type Error = ();
    fn try_into(self) -> Result<kritor::event::EventStructure, Self::Error> {
        let figure_sender = || {
            let mut sender = kritor::common::Sender::default();
            if let Some(user) = self.user {
                sender.uin = user.id.parse().ok();
                sender.nick = user.nick;
                if let Some(member) = self.member {
                    sender.nick = member.nick;
                }
                Some(sender)
            } else {
                None
            }
        };
        match self._type.as_str() {
            "message-created" => Ok(kritor::event::EventStructure {
                r#type: kritor::event::EventType::Message.into(),
                event: Some(kritor::event::event_structure::Event::Message(
                    kritor::common::PushMessageBody {
                        time: (self.timestamp / 1000) as u64,
                        message_seq: 0,
                        contact: self.channel.and_then(|x| x.try_into().ok()),
                        sender: figure_sender(),
                        elements: {
                            match super::message::Parser::new(
                                self.message.as_ref().ok_or(())?.content.as_str(),
                            )
                            .parse()
                            .and_then(|r| r.try_into_kritor_elements())
                            {
                                Ok(r) => r,
                                Err(e) => {
                                    log::error!("Failed to parse message content: {:?}, message set to empty", e);
                                    vec![]
                                }
                            }
                        },
                        message_id: self.message.ok_or(())?.id,
                    },
                )),
            }),
            _ => Err(()),
        }
    }
}

impl TryInto<kritor::common::Contact> for Channel {
    type Error = ();

    fn try_into(self) -> Result<kritor::common::Contact, Self::Error> {
        Ok(kritor::common::Contact {
            scene: match self._type {
                ChannelType::TEXT => kritor::common::Scene::Group,
                ChannelType::DIRECT => kritor::common::Scene::Friend,
                _ => Err(())?,
            }
            .into(),
            peer: if let Some(peer) = self.id.strip_prefix("private:") {
                peer.into()
            } else {
                self.id
            },
            sub_peer: None,
        })
    }
}
