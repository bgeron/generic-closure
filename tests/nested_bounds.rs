use generic_closure::{closure, closure_trait};

closure_trait!(ByteCount<T: AsRef<Vec<u8>>>(value: T) -> usize);
closure_trait!(RowCount<T: AsRef<Vec<Vec<u8>>>>(value: T) -> usize);
closure_trait!(LayerCount<T: AsRef<Vec<Vec<Vec<u8>>>>>(value: T) -> usize);
closure_trait!(InspectRows<T: AsRef<Vec<Vec<u8>>>>(value: T));

#[test]
fn final_generic_bounds_support_arbitrarily_nested_closing_brackets() {
    let byte_count = closure!(
        ByteCount<T: AsRef<Vec<u8>>>(value: T) -> usize { value.as_ref().len() }
    );
    let row_count = closure!(
        RowCount<T: AsRef<Vec<Vec<u8>>>>(value: T) -> usize { value.as_ref().len() }
    );
    let layer_count = closure!(
        LayerCount<T: AsRef<Vec<Vec<Vec<u8>>>>>(value: T) -> usize { value.as_ref().len() }
    );
    let inspect_rows = closure!(
        InspectRows<T: AsRef<Vec<Vec<u8>>>>(value: T) {
            assert_eq!(value.as_ref().len(), 2);
        }
    );

    assert_eq!(byte_count.call(vec![1, 2, 3]), 3);
    assert_eq!(row_count.call(vec![vec![1], vec![2]]), 2);
    assert_eq!(layer_count.call(vec![vec![vec![1]]]), 1);
    inspect_rows.call(vec![vec![1], vec![2]]);
}
