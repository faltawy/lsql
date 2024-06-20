// lsql grammar 
// Manage files with sql lile syntax
lexer grammar LSQLexer;


SELECT : 'SELECT';
FROM : 'FROM';
WHERE : 'WHERE';
CREATE : 'CREATE';
FILE : 'FILE';
CHANGE_DIR : 'CD';

EQ: '=';
LT: '<';
GT: '>';
LPAREN: '(';
RPAREN: ')';
COMMA: ',';
SEMI: ';';
DOT: '.';
DOTDOT: '..';


ID: [a-zA-Z_][a-zA-Z_0-9]*;
NUM: [0-9]+;
WS: [ \t\r\n]+ -> skip;
STRING: '\'' .*? '\'';
