use std::any::Any;

///
/// 旧バージョンのコンフィグを最新版に変換するための仕組み
///

/// 「最新版に変換できる」トレイト
pub trait ConfigMigratable: Any {
    type Latest: Any;

    /// 次のバージョンの型（最新版なら Self を返す）
    fn migrate_boxed(self: Box<Self>) -> Box<dyn ConfigMigratable<Latest = Self::Latest>>;

    /// 今が最新版なら true
    fn is_latest(&self) -> bool;

    /// Any型としてダウンキャストできるようにする
    fn as_any(self: Box<Self>) -> Box<dyn Any>;

    /// 最新版になるまでマイグレーション
    fn migrate_until_latest(self: Box<Self>) -> Self::Latest
    where
        Self: Sized,
    {
        let mut current: Box<dyn ConfigMigratable<Latest = Self::Latest>> = self;
        while !current.is_latest() {
            current = current.migrate_boxed();
        }
        // 最後にdowncast
        *current.as_any().downcast().expect("must be Conf at latest")
    }
}
