import my_rust_module


def test_sum_as_string():
    # Rustの関数が正しく動作し、文字列を返すかテスト
    result = my_rust_module.sum_as_string(5, 7)
    assert result == "12"
    assert isinstance(result, str)
