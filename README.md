# LSQL 
Query Your Files with SQL. 


## supported LSQL commands

- `SELECT` - select files and directories. 
- `FROM` - from a directory.
- `WHERE` - filter files and directories.
- `ORDER BY` - order files and directories.
- `LIMIT` - limit the number of files and directories.
- `DESC` - order in descending order.
- `ASC` - order in ascending order.

## Examples

- `SELECT * FROM /Users/username/Downloads WHERE name = 'file.txt'` -> select file.txt from the Downloads directory.

- `SELECT * WHERE name = 'file.txt' ORDER BY size ASC` -> order by size in ascending order.
