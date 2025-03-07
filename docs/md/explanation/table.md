# Table

`Table` is Perspective's columnar data frame, analogous to a Pandas `DataFrame`
or Apache Arrow, supporting append & in-place updates, removal by index, and
update notifications.

A `Table` contains columns, each of which have a unique name, are strongly and
consistently typed, and contains rows of data conforming to the column's type.
Each column in a `Table` must have the same number of rows, though not every row
must contain data; null-values are used to indicate missing values in the
dataset. The schema of a `Table` is _immutable after creation_, which means the
column names and data types cannot be changed after the `Table` has been
created. Columns cannot be added or deleted after creation either, but a `View`
can be used to select an arbitrary set of columns from the `Table`.
