parser grammar LSQLParser;

options { tokenVocab=LSQLexer; }

sql_script: statement* EOF;

statement: 
    select_statement
    | 
    change_dir_statement
;

column: ID;

value: NUM | STRING;

assignment: column EQ value;

column_list: column (COMMA column)*;

table_name: ID;

condition: column EQ value;


where_clause: WHERE condition;

value_list: value (COMMA value)*;

assignment_list: assignment (COMMA assignment)*;


select_statement: SELECT column_list FROM table_name WHERE where_clause;
change_dir_statement: CHANGE_DIR value;