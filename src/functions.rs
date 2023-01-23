use crate::enums::ActionKind;

pub(crate) fn get_arg_from_function_call(action: &near_indexer::near_primitives::views::ActionView) -> Option<serde_json::Value> {
    let (action_kind, args) =
    crate::serializers::extract_action_type_and_value_from_action_view(action);

    match action_kind {
        ActionKind::FunctionCall => {
            return Some(args);
        },
        _ => None,
    }
}