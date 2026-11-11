use super::{edge_scroll_step, insertion_slot, move_destination};

#[test]
fn insertion_boundaries_include_the_first_last_and_virtual_rows() {
    assert_eq!(insertion_slot(-10., 500), 0);
    assert_eq!(insertion_slot(28., 500), 0);
    assert_eq!(insertion_slot(30., 500), 1);
    assert_eq!(insertion_slot(58. * 200. + 10., 500), 200);
    assert_eq!(insertion_slot(58. * 200. + 40., 500), 201);
    assert_eq!(insertion_slot(100000., 500), 500);
    assert_eq!(insertion_slot(20., 0), 0);
}

#[test]
fn moving_uses_final_indices_and_skips_no_op_boundaries() {
    assert_eq!(move_destination(0, 5, 5), Some(4));
    assert_eq!(move_destination(4, 0, 5), Some(0));
    assert_eq!(move_destination(1, 4, 5), Some(3));
    assert_eq!(move_destination(3, 1, 5), Some(1));
    assert_eq!(move_destination(2, 2, 5), None);
    assert_eq!(move_destination(2, 3, 5), None);
    assert_eq!(move_destination(5, 0, 5), None);
    assert_eq!(move_destination(0, 6, 5), None);
    assert_eq!(move_destination(0, 0, 0), None);
}

#[test]
fn auto_scroll_only_operates_inside_the_edge_zones() {
    assert_eq!(edge_scroll_step(-1., 300.), 0.);
    assert_eq!(edge_scroll_step(301., 300.), 0.);
    assert_eq!(edge_scroll_step(150., 300.), 0.);
    assert_eq!(edge_scroll_step(0., 300.), 10.);
    assert_eq!(edge_scroll_step(300., 300.), -10.);
    assert_eq!(edge_scroll_step(18., 300.), 5.);
    assert_eq!(edge_scroll_step(282., 300.), -5.);
}
