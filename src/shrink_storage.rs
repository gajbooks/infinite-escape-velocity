#[macro_export]
macro_rules! shrink_storage {
    ( $shrink_var:expr ) => {
        if $shrink_var.capacity() > ($shrink_var.len() * 5) && $shrink_var.len() > 10 {
            $shrink_var.shrink_to_fit();
        }
    };
}