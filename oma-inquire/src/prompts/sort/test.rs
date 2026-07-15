use crate::{
    formatter::MultiOptionFormatter,
    list_option::ListOption,
    test::fake_backend,
    ui::{Key, KeyModifiers},
    Sort,
};

#[test]
/// 验证闭包 Formatter 是否能正确拦截并格式化最终的排序结果
fn closure_formatter() {
    // 更改为使用 Enter 提交
    let mut backend = fake_backend(vec![Key::Enter]);

    let formatted = String::from("Custom Sort Finished!");
    let formatter: MultiOptionFormatter<'_, i32> = &|_| formatted.clone();

    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .with_formatter(formatter)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 2),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证默认状态下，不进行任何排序操作直接按 Enter 提交时，顺序保持原样
fn default_no_op_submission() {
    let mut backend = fake_backend(vec![Key::Enter]);
    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 2),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证使用 Space 选中元素后，通过 Down 键向下拖拽元素的效果
fn drag_item_down_via_space_and_arrow() {
    let mut backend = fake_backend(vec![
        Key::Char(' ', KeyModifiers::NONE), // 选中元素 1 (索引 0)
        Key::Down(KeyModifiers::NONE),      // 1 和 2 交换 -> [2, 1, 3], 光标停留在索引 1 (值 1)
        Key::Char(' ', KeyModifiers::NONE), // 放下元素 1
        Key::Enter,                         // 提交
    ]);

    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 2),
        ListOption::new(1, 1),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证下移光标后，再使用 Space 选中元素并向上拖拽的效果
fn move_cursor_and_drag_up() {
    let mut backend = fake_backend(vec![
        Key::Down(KeyModifiers::NONE),      // 光标移到 2 (索引 1)
        Key::Char(' ', KeyModifiers::NONE), // 选中元素 2
        Key::Up(KeyModifiers::NONE),        // 2 和 1 交换 -> [2, 1, 3], 光标随之回到索引 0 (值 2)
        Key::Char(' ', KeyModifiers::NONE), // 放下元素 2
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 2),
        ListOption::new(1, 1),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证 starting_cursor (起始光标) 设置后，Space 拖拽的目标是否正确
fn starting_cursor_is_respected() {
    let mut backend = fake_backend(vec![
        Key::Char(' ', KeyModifiers::NONE), // 选中起始位置的元素 3 (索引 2)
        Key::Up(KeyModifiers::NONE),        // 3 和 2 交换 -> [1, 3, 2], 光标随之移动到索引 1 (值 3)
        Key::Char(' ', KeyModifiers::NONE), // 放下元素 3
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    // 从索引 2 (值为 3) 启动光标
    let ans = Sort::new("Question", options)
        .with_starting_cursor(2)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 3),
        ListOption::new(2, 2),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证在边界处拖拽时（如在最顶端继续向上拖拽）不会发生越界或 Panic，并且保持原有顺序不变
fn edge_drag_does_not_panic_or_corrupt() {
    let mut backend = fake_backend(vec![
        Key::Char(' ', KeyModifiers::NONE), // 选中元素 1 (索引 0)
        Key::Up(KeyModifiers::NONE),        // 已经在顶端，向上拖拽无效
        Key::Char(' ', KeyModifiers::NONE), // 放下元素 1
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 2),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}

#[test]
/// 验证开启 Vim 模式时的按键操作 (j/k 移动光标，Space 选中后 j/k 拖拽元素)
fn vim_mode_navigation_and_drag() {
    let mut backend = fake_backend(vec![
        Key::Char('j', KeyModifiers::NONE), // 光标向下移到 2 (索引 1)
        Key::Char(' ', KeyModifiers::NONE), // 选中元素 2
        Key::Char('j', KeyModifiers::NONE), // 2 向下拖拽 -> [1, 3, 2], 光标随之到索引 2 (值 2)
        Key::Char('k', KeyModifiers::NONE), // 2 向上拖拽 -> [1, 2, 3], 光标随之回到索引 1 (值 2)
        Key::Char(' ', KeyModifiers::NONE), // 放下元素 2
        Key::Enter,
    ]);

    let options = vec![1, 2, 3];

    let ans = Sort::new("Question", options)
        .with_vim_mode(true)
        .prompt_with_backend(&mut backend)
        .unwrap();

    let expected = vec![
        ListOption::new(0, 1),
        ListOption::new(1, 2),
        ListOption::new(2, 3),
    ];
    assert_eq!(expected, ans);
}
