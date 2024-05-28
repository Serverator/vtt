use crate::prelude::*;
use lightyear::prelude::*;

pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
	fn build(&self, app: &mut App) {
		app.add_message::<TextMessage>(ChannelDirection::Bidirectional);
		app.add_channel::<UnorderedReliableChannel>(ChannelSettings {
			mode: ChannelMode::UnorderedReliable(ReliableSettings::default()),
			..default()
		});
	}
}

#[derive(Channel)]
pub struct UnorderedReliableChannel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextMessage(pub String);