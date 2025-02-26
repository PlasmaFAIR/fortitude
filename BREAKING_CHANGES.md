# Breaking Changes

## 0.7.0

We've reorganised a lot of the rules and categories. Rule and category
redirects should minimise any breakages, but you should update your
config appropriately.

We have also changed the rules that are on by default. This may mean
you now have to explicitly enable some rules you were previously
checking, and you may find more warnings being raised.

## 0.6.0

- `external-function` has been renamed to the more accurate
  `procedure-not-in-module`. The rule code is still `M001`.
