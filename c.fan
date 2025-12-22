<start> ::= <compilationUnit> 

<primaryExpression> ::= <Identifier> |    <Constant> |    <StringLiteral> <StringLiteral>* |    '(' <expression> ')' |    <genericSelection> |    <extension_e> '(' <compoundStatement> ')' |    '__builtin_va_arg' '(' <unaryExpression> ',' <typeName> ')' |    '__builtin_offsetof' '(' <typeName> ',' <unaryExpression> ')' 
<genericSelection> ::= '_Generic' '(' <assignmentExpression> ',' <genericAssocList> ')' 
<genericAssocList> ::= <genericAssociation> (',' <genericAssociation>)* 
<genericAssociation> ::= (<typeName> | 'default') ':' <assignmentExpression> 
<postfixExpression> ::= (<primaryExpression> | <extension_e> '(' <typeName> ')' '{' <initializerList> <comma_e> '}') ('[' <expression> ']' | '(' <argumentExpressionList_e> ')' | ('.' | '->') <Identifier> | '++' | '--')* 
<unaryExpression> ::= ('++' | '--' | 'sizeof')* (<unaryOperator> <castExpression> | ('sizeof' | '_Alignof') '(' <typeName> ')' | '&&' <Identifier> | <postfixExpression>) 
<unaryOperator> ::= '&' | '*' | '+' | '-' | '~' | '!' 
<castExpression> ::= <extension_e> '(' <typeName> ')' <castExpression> | <DigitSequence> | <unaryExpression> 
<multiplicativeExpression> ::= <castExpression> (('*' | '/' | '%') <castExpression>)* 
<additiveExpression> ::= <multiplicativeExpression> (('+' | '-') <multiplicativeExpression>)* 

<extension_e> ::= <e> | '__extension__' 

<shiftExpression> ::= <additiveExpression> (('<<' | '>>') <additiveExpression>)* 

<relationalExpression> ::= <shiftExpression> (('<' | '>' | '<=' | '>=') <shiftExpression>)* 

<equalityExpression> ::= <relationalExpression> <equalityExpression_inner>* 
<equalityExpression_inner> ::= ('==' | '!=') <relationalExpression> 

<andExpression> ::= <equalityExpression> ('&' <equalityExpression>)* 
<exclusiveOrExpression> ::= <andExpression> ('^' <andExpression>)* 
<inclusiveOrExpression> ::= <exclusiveOrExpression> ('|' <exclusiveOrExpression>)* 
<logicalAndExpression> ::= <inclusiveOrExpression> ('&&' <inclusiveOrExpression>)* 
<logicalOrExpression> ::= <logicalAndExpression> ('||' <logicalAndExpression>)* 
<conditionalExpression> ::= <logicalOrExpression> <conditionalExpression_inner_e> 
<conditionalExpression_inner_e> ::= <e> | <conditionalExpression_inner> 
<conditionalExpression_inner> ::= '?' <expression> ':' <conditionalExpression> 

<expression_e> ::= <e> | <expression> 
<expression> ::= <assignmentExpression> (',' <assignmentExpression>)* 
<constantExpression> ::= <conditionalExpression> 
<declaration> ::= <declarationSpecifiers> <initDeclaratorList_e> ';' | <staticAssertDeclaration> 
<declarationSpecifiers2> ::= <declarationSpecifier>+ 

<initDeclaratorList_e> ::= <e> | <initDeclaratorList> 
<initDeclaratorList> ::= <initDeclarator> (',' <initDeclarator>)* 
<initDeclarator> ::= <declarator> <initializer_e> 
<initializer_e> ::= '=' <e> | <initializer> 
<structOrUnionSpecifier> ::= <structOrUnion> <Identifier_e> '{' <structDeclarationList> '}' | <structOrUnion> <Identifier> 
<structOrUnion> ::= 'struct' | 'union' 
<structDeclarationList> ::= <structDeclaration>+ 
<structDeclaration> ::= <specifierQualifierList> <structDeclaratorList> ';' | <specifierQualifierList> ';' | <staticAssertDeclaration> 

<structDeclaratorList> ::= <structDeclarator> (',' <structDeclarator>)* 
<structDeclarator> ::= <declarator> | <declarator_e> ':' <constantExpression> 
<enumSpecifier> ::= 'enum' <Identifier_e> '{' <enumeratorList> <comma_e> '}' | 'enum' <Identifier> 
<enumeratorList> ::= <enumerator> (',' <enumerator>)* 
<enumerator> ::= <enumerationConstant> <equalConstantExpression_e> 
<equalConstantExpression_e> ::= <e> | '=' <constantExpression> 
<enumerationConstant> ::= <Identifier> 
<functionSpecifier> ::= 'inline' | '_Noreturn' | '__inline__' | '__stdcall' | <gccAttributeSpecifier> | '__declspec' '(' <Identifier> ')' 
<alignmentSpecifier> ::= '_Alignas' '(' (<typeName> | <constantExpression>) ')' 
<declarator_e> ::= <e> | <declarator> 
<declarator> ::= <pointer_e> <directDeclarator> <gccDeclaratorExtension>* 

<directDeclarator> ::= <Identifier> <directDeclarator_R_e> |    '(' <declarator> ')' <directDeclarator_R_e> |    <Identifier> ':' <DigitSequence> <directDeclarator_R_e> |    <vcSpecificModifer> <Identifier> <directDeclarator_R_e> |    '(' <vcSpecificModifer> <declarator> ')' <directDeclarator_R_e> 

<directDeclarator_R_e> ::= <e> |    '[' <typeQualifierList_e> <assignmentExpression_e> ']' <directDeclarator_R_e> |    '[' 'static' <typeQualifierList_e> <assignmentExpression> ']' <directDeclarator_R_e> |    '[' <typeQualifierList> 'static' <assignmentExpression> ']' <directDeclarator_R_e> |    '[' <typeQualifierList_e> '*' ']' <directDeclarator_R_e> |    '(' <parameterTypeList> ')' <directDeclarator_R_e> |    '(' <identifierList_e> ')' <directDeclarator_R_e> 

<vcSpecificModifer> ::= '__cdecl' | '__clrcall' | '__stdcall' | '__fastcall' | '__thiscall' | '__vectorcall' 
<parameterTypeList_e> ::= <e> | <parameterTypeList> 
<parameterTypeList> ::= <parameterList> <comma_ellipsis_e> 
<comma_ellipsis_e> ::= <e> | ',' '...' 
<parameterList> ::= <parameterDeclaration> (',' <parameterDeclaration>)* 
<parameterDeclaration> ::= <declarationSpecifiers> <declarator> | <declarationSpecifiers2> <abstractDeclarator_e> 
<identifierList_e> ::= <e> | <identifierList> 
<identifierList> ::= <Identifier> (',' <Identifier>)* 
<typedefName> ::= <Identifier> 
<initializer> ::= '{' <initializerList> <comma_e> '}' | <assignmentExpression>
<comma_e> ::= <e> | ',' 
<initializerList> ::= <designation_e> <initializer> (',' <designation_e> <initializer>)* 
<designation_e> ::= <e> | <designation> 
<designation> ::= <designatorList> '=' 
<designatorList> ::= <designator>+ 
<designator> ::= '[' <constantExpression> ']' | '.' <Identifier> 
<staticAssertDeclaration> ::= '_Static_assert' '(' <constantExpression> ',' <StringLiteral>+ ')' ';' 
<statement_e> ::= <e> | <statement> 
<statement> ::= <labeledStatement> | <compoundStatement> | <expressionStatement> | <selectionStatement> | <iterationStatement> | <jumpStatement> | ('__asm' | '__asm__') ('volatile' | '__volatile__') '(' <logicalOrExpressions_e> (':' <logicalOrExpressions_e>)* ')' ';' 
<logicalOrExpressions_e> ::= <e> | (<logicalOrExpression> (',' <logicalOrExpression>)*) 
<labeledStatement> ::= <Identifier> ':' <statement_e> | 'case' <constantExpression> ':' <statement> | 'default' ':' <statement> 
<compoundStatement> ::= '{' <blockItemList_e> '}' 
<blockItemList_e> ::= <e> | <blockItemList> 
<blockItemList> ::= <blockItem>+ 
<blockItem> ::= <statement> | <declaration> 
<expressionStatement> ::= <expression_e> ';' 
<selectionStatement> ::= 'if' '(' <expression> ')' <statement> <else_e> | 'switch' '(' <expression> ')' <statement> 
<else_e> ::= <e> | 'else' <statement> 
<iterationStatement> ::= <While> '(' <expression> ')' <statement> | <Do> <statement> <While> '(' <expression> ')' ';' | <For> '(' <forCondition> ')' <statement> 
<forCondition> ::= (<forDeclaration> | <expression_e>) ';' <forExpression_e> ';' <forExpression_e> 
<forDeclaration> ::= <declarationSpecifiers> <initDeclaratorList_e> 
<forExpression_e> ::= <e> | <forExpression> 
<forExpression> ::= <assignmentExpression> (',' <assignmentExpression>)* 
<jumpStatement> ::= ('goto' <Identifier> | 'continue' | 'break' | 'return' <expression_e> | 'goto' <unaryExpression>) ';' 
<compilationUnit> ::= <translationUnit_e> 
<translationUnit_e> ::= <e> | <translationUnit> 
<translationUnit> ::= <externalDeclaration>+ 
<externalDeclaration> ::= <functionDefinition> | <declaration> | ';' 
<functionDefinition> ::= <declarationSpecifiers_e> <declarator> <declarationList_e> <compoundStatement> 

<declarationSpecifiers_e> ::= <e> | <declarationSpecifiers> 
<declarationSpecifiers> ::= <declarationSpecifier>+ 
<declarationSpecifier> ::= <storageClassSpecifier> | <typeSpecifier> | <typeQualifier> | <functionSpecifier> | <alignmentSpecifier> 
<storageClassSpecifier> ::= 'typedef' | 'extern' | 'static' | '_Thread_local' | 'auto' | 'register' 
<typeSpecifier> ::= 'void' | 'char' | 'short' | 'int' | 'long' | 'float' | 'double' | 'signed' | 'unsigned' | '_Bool' | '_Complex' | '__m128' | '__m128d' | '__m128i' | '__extension__' '(' ('__m128' | '__m128d' | '__m128i') ')' | <atomicTypeSpecifier> | <structOrUnionSpecifier> | <enumSpecifier> | <typedefName> | '__typeof__' '(' <constantExpression> ')' 
<atomicTypeSpecifier> ::= '_Atomic' '(' <typeName> ')' 
<typeName> ::= <specifierQualifierList> <abstractDeclarator_e> 
<abstractDeclarator_e> ::= <e> | <abstractDeclarator> 
<abstractDeclarator> ::= <pointer_e> <directAbstractDeclarator> <gccDeclaratorExtension>* | <pointer> 
<directAbstractDeclarator> ::= '(' <abstractDeclarator> ')' <gccDeclaratorExtension>* <directAbstractDeclarator_R_e> |    '[' <typeQualifierList_e> <assignmentExpression_e> ']' <directAbstractDeclarator_R_e> |    '[' 'static' <typeQualifierList_e> <assignmentExpression> ']' <directAbstractDeclarator_R_e> |    '[' <typeQualifierList> 'static' <assignmentExpression> ']' <directAbstractDeclarator_R_e> |    '[' '*' ']' <directAbstractDeclarator_R_e> |    '(' <parameterTypeList_e> ')' <gccDeclaratorExtension>* <directAbstractDeclarator_R_e> 
<directAbstractDeclarator_R_e> ::= <e> | '[' <typeQualifierList_e> <assignmentExpression_e> ']' <directAbstractDeclarator_R_e> |    '[' 'static' <typeQualifierList_e> <assignmentExpression> ']' <directAbstractDeclarator_R_e> |    '[' <typeQualifierList> 'static' <assignmentExpression> ']' <directAbstractDeclarator_R_e> |    '[' '*' ']' <directAbstractDeclarator_R_e> |    '(' <parameterTypeList_e> ')' <gccDeclaratorExtension>* <directAbstractDeclarator_R_e> 
<gccDeclaratorExtension> ::= '__asm' '(' <StringLiteral>+ ')' | <gccAttributeSpecifier> 
<gccAttributeSpecifier> ::= '__attribute__' '(' '(' <gccAttributeList> ')' ')' 
<gccAttributeList> ::= <gccAttribute_e> (',' <gccAttribute_e>)* 
<gccAttribute_e> ::= <e> | <gccAttribute> 
<gccAttribute> ::= <not_parens_or_comma> <gccAttribute_inner_e> 
<not_parens_or_comma> ::= <alnums> 
<gccAttribute_inner_e> ::= <e> | <gccAttribute_inner> 
<gccAttribute_inner> ::= '(' <argumentExpressionList_e> ')' 
<argumentExpressionList_e> ::= <e> | <argumentExpressionList> 
<argumentExpressionList> ::= <assignmentExpression> (',' <assignmentExpression>)* 

<assignmentExpression_e> ::= <e> | <assignmentExpression> 
<assignmentExpression> ::= <conditionalExpression> | <unaryExpression> <assignmentOperator> <assignmentExpression> | <DigitSequence> 
<assignmentOperator> ::= '=' | '*=' | '/=' | '%=' | '+=' | '-=' | '<<=' | '>>=' | '&=' | '^=' | '|=' 

<pointer_e> ::= <e> | <pointer> 
<pointer> ::= <pointer_inner>+ 
<pointer_inner> ::= <star_or_hat> <typeQualifierList_e> 
<star_or_hat> ::= '*' | '^' 
<typeQualifierList_e> ::= <e> | <typeQualifierList> 
<typeQualifierList> ::= <typeQualifier>+ 
<specifierQualifierList_e> ::= <e> | <specifierQualifierList> 
<specifierQualifierList> ::= (<typeSpecifier> | <typeQualifier>) <specifierQualifierList_e> 
<typeQualifier> ::= 'const' | 'restrict' | 'volatile' | '_Atomic' 

<declarationList_e> ::= <e> | <declarationList> 
<declarationList> ::= <declaration>+ 
<Auto> ::= 'auto' 
<Break> ::= 'break' 
<Case> ::= 'case' 
<Char> ::= 'char' 
<Const> ::= 'const' 
<Continue> ::= 'continue' 
<Default> ::= 'default' 
<Do> ::= 'do' 
<Double> ::= 'double' 
<Else> ::= 'else' 
<Enum> ::= 'enum' 
<Extern> ::= 'extern' 
<Float> ::= 'float' 
<For> ::= 'for' 
<Goto> ::= 'goto' 
<If> ::= 'if' 
<Inline> ::= 'inline' 
<Int> ::= 'int' 
<Long> ::= 'long' 
<Register> ::= 'register' 
<Restrict> ::= 'restrict' 
<Return> ::= 'return' 
<Short> ::= 'short' 
<Signed> ::= 'signed' 
<Sizeof> ::= 'sizeof' 
<Static> ::= 'static' 
<Struct> ::= 'struct' 
<Switch> ::= 'switch' 
<Typedef> ::= 'typedef' 
<Union> ::= 'union' 
<Unsigned> ::= 'unsigned' 
<Void> ::= 'void' 
<Volatile> ::= 'volatile' 
<While> ::= 'while' 
<Alignas> ::= '_Alignas' 
<Alignof> ::= '_Alignof' 
<Atomic> ::= '_Atomic' 
<Bool> ::= '_Bool' 
<Complex> ::= '_Complex' 
<Generic> ::= '_Generic' 
<Imaginary> ::= '_Imaginary' 
<Noreturn> ::= '_Noreturn' 
<StaticAssert> ::= '_Static_assert' 
<ThreadLocal> ::= '_Thread_local' 
<LeftParen> ::= '(' 
<RightParen> ::= ')' 
<LeftBracket> ::= '[' 
<RightBracket> ::= ']' 
<LeftBrace> ::= '{' 
<RightBrace> ::= '}' 
<Less> ::= '<' 
<LessEqual> ::= '<=' 
<Greater> ::= '>' 
<GreaterEqual> ::= '>=' 
<LeftShift> ::= '<<' 
<RightShift> ::= '>>' 
<PlusPlus> ::= '++' 
<Minus> ::= '-' 
<MinusMinus> ::= '--' 
<Div> ::= '/' 
<Mod> ::= '%' 
<And> ::= '&' 
<Or> ::= '|' 
<AndAnd> ::= '&&' 
<OrOr> ::= '||' 
<Caret> ::= '^' 
<Not> ::= '!' 
<Tilde> ::= '~' 
<Question> ::= '?' 
<Colon> ::= ':' 
<Semi> ::= ';' 
<Comma> ::= ',' 
<Assign> ::= '=' 
<StarAssign> ::= '*=' 
<DivAssign> ::= '/=' 
<ModAssign> ::= '%=' 
<PlusAssign> ::= '+=' 
<MinusAssign> ::= '-=' 
<LeftShiftAssign> ::= '<<=' 
<RightShiftAssign> ::= '>>=' 
<AndAssign> ::= '&=' 
<XorAssign> ::= '^=' 
<OrAssign> ::= '|=' 
<Equal> ::= '==' 
<NotEqual> ::= '!=' 
<Arrow> ::= '->' 
<Dot> ::= '.' 
<Ellipsis> ::= '...' 
<Identifier_e> ::= <e> | <Identifier> 
<Identifier> ::= <IdentifierNondigit> (<IdentifierNondigit> | <Digit>)* 
<IdentifierNondigit> ::= <Nondigit> | <UniversalCharacterName> 
<Nondigit> ::= <letter_or_underscore> 
<Constant> ::= <IntegerConstant> | <FloatingConstant> | <CharacterConstant> 
<IntegerConstant> ::= <DecimalConstant> <IntegerSuffix_e> | <OctalConstant> <IntegerSuffix_e> | <HexadecimalConstant> <IntegerSuffix_e> | <BinaryConstant> 
<BinaryConstant> ::= '0' ('b' | 'B') <binary>+ 
<DecimalConstant> ::= <NonzeroDigit> <Digit>* 
<OctalConstant> ::= '0' <OctalDigit>* 
<HexadecimalConstant> ::= <HexadecimalPrefix> <HexadecimalDigit>+ 
<IntegerSuffix_e> ::= <e> | <IntegerSuffix> 
<IntegerSuffix> ::= <UnsignedSuffix> <LongSuffix_e> | <UnsignedSuffix> <LongLongSuffix> | <LongSuffix> <UnsignedSuffix_e> | <LongLongSuffix> <UnsignedSuffix_e> 
<UnsignedSuffix_e> ::= <e> | <UnsignedSuffix> 
<UnsignedSuffix> ::= 'u' | 'U' 
<LongSuffix_e> ::= <e> | <LongSuffix> 
<LongSuffix> ::= 'l' | 'L' 
<LongLongSuffix> ::= 'll' | 'LL' 
<FloatingConstant> ::= <DecimalFloatingConstant> | <HexadecimalFloatingConstant> 
<DecimalFloatingConstant> ::= <FractionalConstant> <ExponentPart_e> <FloatingSuffix_e> | <DigitSequence> <ExponentPart> <FloatingSuffix_e> 
<HexadecimalFloatingConstant> ::= <HexadecimalPrefix> (<HexadecimalFractionalConstant> | <HexadecimalDigitSequence>) <BinaryExponentPart> <FloatingSuffix_e> 
<FractionalConstant> ::= <DigitSequence_e> '.' <DigitSequence> | <DigitSequence> '.' 
<ExponentPart_e> ::= <e> | <ExponentPart> 
<ExponentPart> ::= ('e' | 'E') <Sign_e> <DigitSequence> 
<DigitSequence_e> ::= <e> | <DigitSequence> 
<DigitSequence> ::= <Digit> <Digit>* 
<HexadecimalFractionalConstant> ::= <HexadecimalDigitSequence_e> '.' <HexadecimalDigitSequence> | <HexadecimalDigitSequence> '.' 
<BinaryExponentPart> ::= ('p' | 'P') <Sign_e> <DigitSequence> 
<Sign_e> ::= <e> | <Sign> 
<Sign> ::= '+' | '-' 
<HexadecimalDigitSequence_e> ::= <e> | <HexadecimalDigitSequence> 
<HexadecimalDigitSequence> ::= <HexadecimalDigit>+ 
<FloatingSuffix_e> ::= <e> | <FloatingSuffix> 
<FloatingSuffix> ::= 'f' | 'l' | 'F' | 'L' 
<CharacterConstant> ::= '\'' <CCharSequence> '\'' | 'L\'' <CCharSequence> '\'' | 'u\'' <CCharSequence> '\'' | 'U\'' <CCharSequence> '\'' 
<CCharSequence> ::= <CChar>+ 
<CChar> ::= <char_chars> | <EscapeSequence> 
<StringLiteral> ::= <EncodingPrefix_e> '"' <SCharSequence_e> '"' 
<EncodingPrefix_e> ::= <e> | <EncodingPrefix> 
<EncodingPrefix> ::= 'u8' | 'u' | 'U' | 'L' 
<SCharSequence_e> ::= <e> | <SCharSequence> 
<SCharSequence> ::= <SChar>+ 
<UniversalCharacterName> ::= '\\u' <HexQuad> | '\\U' <HexQuad> <HexQuad> 
<EscapeSequence> ::= <SimpleEscapeSequence> | <OctalEscapeSequence> | <HexadecimalEscapeSequence> | <UniversalCharacterName> 
<SChar> ::= <alnum> | <EscapeSequence> | '\\\n' | '\\\r\n' 
<HexQuad> ::= <HexadecimalDigit> <HexadecimalDigit> <HexadecimalDigit> <HexadecimalDigit> 
<HexadecimalEscapeSequence> ::= '\\x' <HexadecimalDigit>+ 
<OctalEscapeSequence> ::= '\\' <OctalDigit> <OctalDigit_e> <OctalDigit_e> 
<OctalDigit_e> ::= <e> | <OctalDigit> 
<HexadecimalPrefix> ::= '0x' | '0X' 
<NonzeroDigit> ::= <nonzero_digit> 
<OctalDigit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" 
<HexadecimalDigit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "a" | "b" | "c" | "d" | "e" | "f" 
<SimpleEscapeSequence> ::= "\\'" | '\\"' | '\\?' | '\\a' | '\\b' | '\\f' | '\\n' | '\\r' | '\\t' | '\\v' | '\\\\' 
<binary> ::= "0" | "1" 
<alnums> ::= <alnum> | <alnum> <alnums> 
<alnum> ::= <letter> | <digit> 
<letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" 
<letter_or_underscore> ::= "_" | <letter> 
<Digit> ::= <digit> 
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" 
<nonzero_digit> ::= "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" 
<char_chars> ::= <alnum> 

<e> ::= ' ' ;