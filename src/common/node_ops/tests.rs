use crate::common::node_ops::get_xname_from_user_nid_expression;

#[test]
fn test_get_xname_from_user_nid_expression() {
    let user_input_nid = "nid000001,  nid000002";

    let nid_available_vec = vec![1, 2, 3, 4];

    let sol =
        get_xname_from_user_nid_expression(&user_input_nid, false, &nid_available_vec).unwrap();

    assert_eq!(sol, vec![1, 2])
}

#[test]
fn test_get_xname_from_user_nid_expression_2() {
    let user_input_nid = "nid00000[0,1],  nid000002";

    let nid_available_vec = vec![0, 1, 2, 3, 4];

    let sol =
        get_xname_from_user_nid_expression(&user_input_nid, false, &nid_available_vec).unwrap();

    assert_eq!(sol, vec![0, 1, 2])
}

#[test]
fn test_get_xname_from_user_nid_expression_3() {
    let user_input_nid = "nid00000.*,  nid000010";

    let nid_available_vec = vec![0, 1, 2, 3, 4, 10];

    let sol =
        get_xname_from_user_nid_expression(&user_input_nid, true, &nid_available_vec).unwrap();

    assert_eq!(sol, vec![0, 1, 2, 3, 4, 10])
}

#[test]
#[should_panic(expected = "Nid 'nid0001' not valid, Nid does not have 9 characters")]
fn test_get_xname_from_user_nid_expression_4() {
    let user_input_nid = "nid0001,   nid0002";

    let nid_available_vec = vec![0, 1, 2, 3, 4, 10];

    let _ = get_xname_from_user_nid_expression(&user_input_nid, false, &nid_available_vec).unwrap();
}

#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: Message(\"regex parse error:\\n    nid00000[0\\n            ^\\nerror: unclosed character class\""
)]
fn test_get_xname_from_user_nid_expression_5() {
    let user_input_nid = "nid00000[0,1],  nid000002";

    let nid_available_vec = vec![0, 1, 2, 3, 4];

    let _ = get_xname_from_user_nid_expression(&user_input_nid, true, &nid_available_vec).unwrap();
}
