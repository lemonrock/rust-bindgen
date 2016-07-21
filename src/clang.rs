#![allow(non_upper_case_globals, dead_code)]

use std::os::raw::{c_uint, c_char, c_int, c_ulong, c_longlong};
use std::{mem, ptr};
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ffi::{CString, CStr};

use clang_sys::*;
use cexpr::token::{Token as CexprToken,Kind as CexprTokenKind};

// Cursor
#[derive(Copy, Clone)]
pub struct Cursor {
    x: CXCursor
}

pub type CursorVisitor<'s> = for<'a, 'b> FnMut(&'a Cursor, &'b Cursor) -> CXChildVisitResult + 's;

impl Cursor {
    // common
    pub fn spelling(&self) -> String {
        unsafe {
            String_ { x: clang_getCursorSpelling(self.x) }.to_string()
        }
    }

    pub fn display_name(&self) -> String {
        unsafe {
            String_ { x: clang_getCursorDisplayName(self.x) }.to_string()
        }
    }

    pub fn mangling(&self) -> String {
         unsafe {
            String_ { x: clang_Cursor_getMangling(self.x) }.to_string()
        }
    }

    pub fn lexical_parent(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getCursorLexicalParent(self.x) }
        }
    }

    pub fn semantic_parent(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getCursorSemanticParent(self.x) }
        }
    }

    pub fn kind(&self) -> CXCursorKind {
        unsafe {
            clang_getCursorKind(self.x)
        }
    }

    pub fn is_template(&self) -> bool {
        self.specialized().is_valid()
    }

    pub fn is_valid(&self) -> bool {
        unsafe {
            clang_isInvalid(self.kind()) == 0
        }
    }

    pub fn location(&self) -> SourceLocation {
        unsafe {
            SourceLocation { x: clang_getCursorLocation(self.x) }
        }
    }

    pub fn extent(&self) -> CXSourceRange {
        unsafe {
            clang_getCursorExtent(self.x)
        }
    }

    pub fn raw_comment(&self) -> String {
        unsafe {
            String_ { x: clang_Cursor_getRawCommentText(self.x) }.to_string()
        }
    }

    pub fn comment(&self) -> Comment {
        unsafe {
            Comment { x: clang_Cursor_getParsedComment(self.x) }
        }
    }

    pub fn cur_type(&self) -> Type {
        unsafe {
            Type { x: clang_getCursorType(self.x) }
        }
    }

    pub fn definition(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getCursorDefinition(self.x) }
        }
    }

    pub fn referenced(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getCursorReferenced(self.x) }
        }
    }

    pub fn canonical(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getCanonicalCursor(self.x) }
        }
    }

    pub fn specialized(&self) -> Cursor {
        unsafe {
           Cursor { x: clang_getSpecializedCursorTemplate(self.x) }
        }
    }

    pub fn visit<F>(&self, func:F)
        where F: for<'a, 'b> FnMut(&'a Cursor, &'b Cursor) -> CXChildVisitResult
    {
        let mut data: Box<CursorVisitor> = Box::new(func);
        let opt_visit = visit_children as extern "C" fn(CXCursor, CXCursor, CXClientData) -> CXChildVisitResult;
        unsafe {
            clang_visitChildren(self.x, opt_visit, mem::transmute(&mut data));
        }
    }

    #[cfg(not(feature="llvm_stable"))]
    pub fn is_inlined_function(&self) -> bool {
        unsafe { clang_Cursor_isFunctionInlined(self.x) != 0 }
    }

    // TODO: Remove this when LLVM 3.9 is released.
    //
    // This is currently used for CI purposes.
    #[cfg(feature="llvm_stable")]
    pub fn is_inlined_function(&self) -> bool {
        false
    }

    // bitfield
    pub fn bit_width(&self) -> Option<u32> {
        unsafe {
            let w = clang_getFieldDeclBitWidth(self.x);
            if w == -1 {
                None
            } else {
                Some(w as u32)
            }
        }
    }

    // enum
    pub fn enum_type(&self) -> Type {
        unsafe {
            Type { x: clang_getEnumDeclIntegerType(self.x) }
        }
    }

    pub fn enum_val(&self) -> i64 {
        unsafe {
            clang_getEnumConstantDeclValue(self.x) as i64
        }
    }

    // typedef
    pub fn typedef_type(&self) -> Type {
        unsafe {
            Type { x: clang_getTypedefDeclUnderlyingType(self.x) }
        }
    }

    // function, variable
    pub fn linkage(&self) -> CXLinkageKind {
        unsafe {
            clang_getCursorLinkage(self.x)
        }
    }

    pub fn visibility(&self) -> CXVisibilityKind {
        unsafe {
            clang_getCursorVisibility(self.x)
        }
    }

    // function
    pub fn args(&self) -> Vec<Cursor> {
        unsafe {
            let num = self.num_args() as usize;
            let mut args = vec!();
            for i in 0..num {
                args.push(Cursor { x: clang_Cursor_getArgument(self.x, i as c_uint) });
            }
            args
        }
    }

    pub fn ret_type(&self) -> Type {
        unsafe {
            Type { x: clang_getCursorResultType(self.x) }
        }
    }

    pub fn num_args(&self) -> i32 {
        unsafe {
            clang_Cursor_getNumArguments(self.x)
        }
    }

    // CXX member
    pub fn access_specifier(&self) -> CX_CXXAccessSpecifier {
        unsafe {
            clang_getCXXAccessSpecifier(self.x)
        }
    }

    pub fn is_mutable_field(&self) -> bool {
        unsafe {
            clang_CXXField_isMutable(self.x) != 0
        }
    }

    // CXX method
    pub fn method_is_static(&self) -> bool {
        unsafe {
            clang_CXXMethod_isStatic(self.x) != 0
        }
    }

    pub fn method_is_virtual(&self) -> bool {
        unsafe {
            clang_CXXMethod_isVirtual(self.x) != 0
        }
    }

    // CXX base
    pub fn is_virtual_base(&self) -> bool {
        unsafe {
            clang_isVirtualBase(self.x) != 0
        }
    }

    // CXX template
    pub fn template_arg_kind(&self, i: c_int) -> CXTemplateArgumentKind {
        unsafe {
            clang_Cursor_getTemplateArgumentKind(self.x, i as c_uint)
        }
    }

    pub fn template_arg_value(&self, i: c_int) -> c_longlong {
        unsafe {
            clang_Cursor_getTemplateArgumentValue(self.x, i as c_uint)
        }
    }
}

extern fn visit_children(cur: CXCursor, parent: CXCursor,
                         data: CXClientData) -> CXChildVisitResult {
    let func: &mut Box<CursorVisitor> = unsafe { mem::transmute(data) };
    (*func)(&Cursor { x : cur }, &Cursor { x: parent })
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        unsafe {
            clang_equalCursors(self.x, other.x) == 1
        }
    }

    fn ne(&self, other: &Cursor) -> bool {
        !self.eq(other)
    }
}

impl Eq for Cursor {}

impl Hash for Cursor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.kind.hash(state);
        self.x.xdata.hash(state);
        self.x.data[0].hash(state);
        self.x.data[1].hash(state);
        self.x.data[2].hash(state);
    }
}

// type
#[derive(Debug)]
pub struct Type {
    x: CXType
}

impl Type {
    // common
    pub fn kind(&self) -> CXTypeKind {
        self.x.kind
    }

    pub fn declaration(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getTypeDeclaration(self.x) }
        }
    }

    pub fn spelling(&self) -> String {
        unsafe {
            String_ { x: clang_getTypeSpelling(self.x) }.to_string()
        }
    }


    // XXX make it more consistent
    //
    // This is currently only used to detect typedefs,
    // so it should not be a problem.
    pub fn sanitized_spelling(&self) -> String {
        self.spelling()
            .replace("const ", "")
            .split(' ').next().unwrap_or("").to_owned()
    }

    pub fn sanitized_spelling_in(&self, possible: &[String]) -> bool {
        let type_spelling = self.sanitized_spelling();
        possible.iter()
                .any(|spelling| *spelling == *type_spelling)
    }

    pub fn is_const(&self) -> bool {
        unsafe {
            clang_isConstQualifiedType(self.x) == 1
        }
    }

    pub fn size(&self) -> usize {
        unsafe {
            let val = clang_Type_getSizeOf(self.x);
            if val < 0 { 0 } else { val as usize }
        }
    }

    pub fn fallible_size(&self) -> Result<usize, CXTypeLayoutError> {
        let val = unsafe { clang_Type_getSizeOf(self.x) };
        if val < 0 {
            Err(Self::err_from_ret(val as i32))
        } else {
            Ok(val as usize)
        }
    }

    fn err_from_ret(val: i32) -> CXTypeLayoutError {
        // CXTypeLayoutError doesn't implement From<i32>
        assert!(val < 0);
        match val {
            -1 => CXTypeLayoutError::Invalid,
            -2 => CXTypeLayoutError::Incomplete,
            -3 => CXTypeLayoutError::Dependent,
            -4 => CXTypeLayoutError::NotConstantSize,
            -5 => CXTypeLayoutError::InvalidFieldName,
            _ => unreachable!(),
        }
    }

    pub fn align(&self) -> usize {
        unsafe {
            let val = clang_Type_getAlignOf(self.x);
            if val < 0 { 0 } else { val as usize }
        }
    }

    pub fn num_template_args(&self) -> c_int {
        unsafe {
            clang_Type_getNumTemplateArguments(self.x)
        }
    }

    pub fn template_arg_type(&self, i: c_uint) -> Type {
        unsafe {
            Type { x: clang_Type_getTemplateArgumentAsType(self.x, i) }
        }
    }

    // pointer
    pub fn pointee_type(&self) -> Type {
        unsafe {
            Type { x: clang_getPointeeType(self.x) }
        }
    }

    // array
    pub fn elem_type(&self) -> Type {
        unsafe {
            Type { x: clang_getArrayElementType(self.x) }
        }
    }

    pub fn array_size(&self) -> usize {
        unsafe {
            clang_getArraySize(self.x) as usize
        }
    }

    // typedef
    pub fn canonical_type(&self) -> Type {
        unsafe {
            Type { x: clang_getCanonicalType(self.x) }
        }
    }

    // function
    pub fn is_variadic(&self) -> bool {
        unsafe {
            clang_isFunctionTypeVariadic(self.x) == 1
        }
    }

    pub fn arg_types(&self) -> Vec<Type> {
        unsafe {
            let num = clang_getNumArgTypes(self.x) as usize;
            let mut args = vec!();
            for i in 0..num {
                args.push(Type { x: clang_getArgType(self.x, i as c_uint) });
            }
            args
        }
    }

    pub fn ret_type(&self) -> Type {
        unsafe {
            Type { x: clang_getResultType(self.x) }
        }
    }

    pub fn call_conv(&self) -> CXCallingConv {
        unsafe {
            clang_getFunctionTypeCallingConv(self.x)
        }
    }

    #[cfg(not(feature="llvm_stable"))]
    pub fn named(&self) -> Type {
        unsafe {
            Type { x: clang_Type_getNamedType(self.x) }
        }
    }
}

// SourceLocation
pub struct SourceLocation {
    x: CXSourceLocation
}

impl SourceLocation {
    pub fn location(&self) -> (File, usize, usize, usize) {
        unsafe {
            let mut file = CXFile(ptr::null_mut());
            let mut line = 0;
            let mut col = 0;
            let mut off = 0;
            clang_getSpellingLocation(self.x, &mut file, &mut line, &mut col, &mut off);
            (File { x: file }, line as usize, col as usize, off as usize)
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (file, line, col, _) = self.location();
        if let Some(name) = file.name() {
            write!(f, "{}:{}:{}", name, line, col)
        } else {
            "builtin definitions".fmt(f)
        }
    }
}

// Comment
pub struct Comment {
    x: CXComment
}

impl Comment {
    pub fn kind(&self) -> CXCommentKind {
        unsafe {
            clang_Comment_getKind(self.x)
        }
    }

    pub fn num_children(&self) -> c_uint {
        unsafe {
            clang_Comment_getNumChildren(self.x)
        }
    }

    pub fn get_child(&self, idx: c_uint) -> Comment {
        unsafe {
            Comment { x: clang_Comment_getChild(self.x, idx) }
        }
    }

    // HTML
    pub fn get_tag_name(&self) -> String {
        unsafe {
            String_ { x: clang_HTMLTagComment_getTagName(self.x) }.to_string()
        }
    }

    pub fn get_num_tag_attrs(&self) -> c_uint {
        unsafe {
            clang_HTMLStartTag_getNumAttrs(self.x)
        }
    }

    pub fn get_tag_attr_name(&self, idx: c_uint) -> String {
        unsafe {
            String_ { x: clang_HTMLStartTag_getAttrName(self.x, idx) }.to_string()
        }
    }

    pub fn get_tag_attr_value(&self, idx: c_uint) -> String {
        unsafe {
            String_ { x: clang_HTMLStartTag_getAttrValue(self.x, idx) }.to_string()
        }
    }
}

// File
pub struct File {
    x: CXFile
}

impl File {
    pub fn name(&self) -> Option<String> {
        if self.x.0.is_null() {
            return None;
        }
        unsafe {
            Some(String_ { x: clang_getFileName(self.x) }.to_string())
        }
    }
}

// String
pub struct String_ {
    x: CXString
}

impl fmt::Display for String_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.x.data.is_null() {
            return "".fmt(f);
        }
        unsafe {
            let c_str = clang_getCString(self.x) as *const c_char;
            let p = c_str as *const _;
            f.write_str(&String::from_utf8_lossy(CStr::from_ptr(p).to_bytes()))
        }
    }
}

// Index
pub struct Index {
    x: CXIndex
}

impl Index {
    pub fn create(pch: bool, diag: bool) -> Index {
        unsafe {
            Index { x: clang_createIndex(pch as c_int, diag as c_int) }
        }
    }

    pub fn dispose(&self) {
        unsafe {
            clang_disposeIndex(self.x);
        }
    }

    pub fn is_null(&self) -> bool {
        self.x.0.is_null()
    }
}

// Token
pub struct Token {
    pub kind: CXTokenKind,
    pub spelling: String,
}

impl Into<CexprToken> for Token {
    fn into(self) -> CexprToken {
        CexprToken {
            kind:match self.kind {
                CXTokenKind::Comment => CexprTokenKind::Comment,
                CXTokenKind::Identifier => CexprTokenKind::Identifier,
                CXTokenKind::Keyword => CexprTokenKind::Keyword,
                CXTokenKind::Literal => CexprTokenKind::Literal,
                CXTokenKind::Punctuation => CexprTokenKind::Punctuation,
            },
            raw:self.spelling.into_bytes().into_boxed_slice()
        }
    }
}

// TranslationUnit
pub struct TranslationUnit {
    x: CXTranslationUnit
}

impl TranslationUnit {
    pub fn parse(ix: &Index, file: &str, cmd_args: &[String],
                 unsaved: &[UnsavedFile], opts: CXTranslationUnit_Flags) -> TranslationUnit {
        let fname = CString::new(file).unwrap();
        let _c_args: Vec<CString> = cmd_args.iter().map(|s| CString::new(s.clone()).unwrap()).collect();
        let c_args: Vec<*const c_char> = _c_args.iter().map(|s| s.as_ptr()).collect();
        let mut c_unsaved: Vec<CXUnsavedFile> = unsaved.iter().map(|f| f.x).collect();
        let tu = unsafe {
            clang_parseTranslationUnit(ix.x, fname.as_ptr(),
                                       c_args.as_ptr(),
                                       c_args.len() as c_int,
                                       c_unsaved.as_mut_ptr(),
                                       c_unsaved.len() as c_uint,
                                       opts)
        };
        TranslationUnit { x: tu }
    }

    pub fn reparse(&self, unsaved: &[UnsavedFile], opts: CXReparse_Flags) -> bool {
        let mut c_unsaved: Vec<CXUnsavedFile> = unsaved.iter().map(|f| f.x).collect();

        unsafe {
            clang_reparseTranslationUnit(self.x,
                                         c_unsaved.len() as c_uint,
                                         c_unsaved.as_mut_ptr(),
                                         opts) == CXErrorCode::Success
        }
    }

    pub fn diags(&self) -> Vec<Diagnostic> {
        unsafe {
            let num = clang_getNumDiagnostics(self.x) as usize;
            let mut diags = vec!();
            for i in 0..num {
                diags.push(Diagnostic { x: clang_getDiagnostic(self.x, i as c_uint) });
            }
            diags
        }
    }

    pub fn cursor(&self) -> Cursor {
        unsafe {
            Cursor { x: clang_getTranslationUnitCursor(self.x) }
        }
    }

    pub fn dispose(&self) {
        unsafe {
            clang_disposeTranslationUnit(self.x);
        }
    }

    pub fn is_null(&self) -> bool {
        self.x.0.is_null()
    }

    pub fn tokens(&self, cursor: &Cursor) -> Option<Vec<Token>> {
        let range = cursor.extent();
        let mut tokens = vec![];
        unsafe {
            let mut token_ptr = ::std::ptr::null_mut();
            let mut num_tokens : c_uint = 0;
            clang_tokenize(self.x, range, &mut token_ptr, &mut num_tokens);
            if token_ptr.is_null() {
                return None;
            }
            let token_array = ::std::slice::from_raw_parts(token_ptr, num_tokens as usize);
            for &token in token_array.iter() {
                let kind = clang_getTokenKind(token);
                let spelling = String_ { x: clang_getTokenSpelling(self.x, token) }.to_string();
                tokens.push(Token { kind: kind, spelling: spelling });
            }
            clang_disposeTokens(self.x, token_ptr, num_tokens);
        }
        Some(tokens)
    }
}

// Diagnostic
pub struct Diagnostic {
    x: CXDiagnostic
}

impl Diagnostic {
    pub fn default_opts() -> CXDiagnosticDisplayOptions {
        unsafe {
            clang_defaultDiagnosticDisplayOptions()
        }
    }

    pub fn format(&self, opts: CXDiagnosticDisplayOptions) -> String {
        unsafe {
            String_ { x: clang_formatDiagnostic(self.x, opts) }.to_string()
        }
    }

    pub fn severity(&self) -> CXDiagnosticSeverity {
        unsafe {
            clang_getDiagnosticSeverity(self.x)
        }
    }

    pub fn dispose(&self) {
        unsafe {
            clang_disposeDiagnostic(self.x);
        }
    }
}

// UnsavedFile
pub struct UnsavedFile {
    x: CXUnsavedFile,
    name: CString,
    contents: CString
}

impl UnsavedFile {
    pub fn new(name: &str, contents: &str) -> UnsavedFile {
        let name = CString::new(name).unwrap();
        let contents = CString::new(contents).unwrap();
        let x = CXUnsavedFile {
            Filename: name.as_ptr(),
            Contents: contents.as_ptr(),
            Length: contents.as_bytes().len() as c_ulong,
        };
        UnsavedFile {
            x: x,
            name: name,
            contents: contents
        }
    }
}

pub fn kind_to_str(x: CXCursorKind) -> &'static str {
    match x {
        CXCursorKind::UnexposedDecl => "UnexposedDecl",
        CXCursorKind::StructDecl => "StructDecl",
        CXCursorKind::UnionDecl => "UnionDecl",
        CXCursorKind::ClassDecl => "ClassDecl",
        CXCursorKind::EnumDecl => "EnumDecl",
        CXCursorKind::FieldDecl => "FieldDecl",
        CXCursorKind::EnumConstantDecl => "EnumConstantDecl",
        CXCursorKind::FunctionDecl => "FunctionDecl",
        CXCursorKind::VarDecl => "VarDecl",
        CXCursorKind::ParmDecl => "ParmDecl",
        CXCursorKind::ObjCInterfaceDecl => "ObjCInterfaceDecl",
        CXCursorKind::ObjCCategoryDecl => "ObjCCategoryDecl",
        CXCursorKind::ObjCProtocolDecl => "ObjCProtocolDecl",
        CXCursorKind::ObjCPropertyDecl => "ObjCPropertyDecl",
        CXCursorKind::ObjCIvarDecl => "ObjCIvarDecl",
        CXCursorKind::ObjCInstanceMethodDecl => "ObjCInstanceMethodDecl",
        CXCursorKind::ObjCClassMethodDecl => "ObjCClassMethodDecl",
        CXCursorKind::ObjCImplementationDecl => "ObjCImplementationDecl",
        CXCursorKind::ObjCCategoryImplDecl => "ObjCCategoryImplDecl",
        CXCursorKind::TypedefDecl => "TypedefDecl",
        CXCursorKind::CXXMethod => "CXXMethod",
        CXCursorKind::Namespace => "Namespace",
        CXCursorKind::LinkageSpec => "LinkageSpec",
        CXCursorKind::Constructor => "Constructor",
        CXCursorKind::Destructor => "Destructor",
        CXCursorKind::ConversionFunction => "ConversionFunction",
        CXCursorKind::TemplateTypeParameter => "TemplateTypeParameter",
        CXCursorKind::NonTypeTemplateParameter => "NonTypeTemplateParameter",
        CXCursorKind::TemplateTemplateParameter => "TemplateTemplateParameter",
        CXCursorKind::FunctionTemplate => "FunctionTemplate",
        CXCursorKind::ClassTemplate => "ClassTemplate",
        CXCursorKind::ClassTemplatePartialSpecialization => "ClassTemplatePartialSpecialization",
        CXCursorKind::NamespaceAlias => "NamespaceAlias",
        CXCursorKind::UsingDirective => "UsingDirective",
        CXCursorKind::UsingDeclaration => "UsingDeclaration",
        CXCursorKind::TypeAliasDecl => "TypeAliasDecl",
        CXCursorKind::ObjCSynthesizeDecl => "ObjCSynthesizeDecl",
        CXCursorKind::ObjCDynamicDecl => "ObjCDynamicDecl",
        CXCursorKind::CXXAccessSpecifier => "CXXAccessSpecifier",
        CXCursorKind::ObjCProtocolRef => "ObjCProtocolRef",
        CXCursorKind::ObjCClassRef => "ObjCClassRef",
        CXCursorKind::TypeRef => "TypeRef",
        CXCursorKind::CXXBaseSpecifier => "CXXBaseSpecifier",
        CXCursorKind::TemplateRef => "TemplateRef",
        CXCursorKind::NamespaceRef => "NamespaceRef",
        CXCursorKind::MemberRef => "MemberRef",
        CXCursorKind::OverloadedDeclRef => "OverloadedDeclRef",
        CXCursorKind::VariableRef => "VariableRef",
        CXCursorKind::NoDeclFound => "NoDeclFound",
        CXCursorKind::NotImplemented => "NotImplemented",
        CXCursorKind::InvalidCode => "InvalidCode",
        CXCursorKind::DeclRefExpr => "DeclRefExpr",
        CXCursorKind::MemberRefExpr => "MemberRefExpr",
        CXCursorKind::CallExpr => "CallExpr",
        CXCursorKind::ObjCMessageExpr => "ObjCMessageExpr",
        CXCursorKind::BlockExpr => "BlockExpr",
        CXCursorKind::IntegerLiteral => "IntegerLiteral",
        CXCursorKind::FloatingLiteral => "FloatingLiteral",
        CXCursorKind::ImaginaryLiteral => "ImaginaryLiteral",
        CXCursorKind::StringLiteral => "StringLiteral",
        CXCursorKind::CharacterLiteral => "CharacterLiteral",
        CXCursorKind::ParenExpr => "ParenExpr",
        CXCursorKind::UnaryOperator => "UnaryOperator",
        CXCursorKind::ArraySubscriptExpr => "ArraySubscriptExpr",
        CXCursorKind::BinaryOperator => "BinaryOperator",
        CXCursorKind::CompoundAssignOperator => "CompoundAssignOperator",
        CXCursorKind::ConditionalOperator => "ConditionalOperator",
        CXCursorKind::CStyleCastExpr => "CStyleCastExpr",
        CXCursorKind::CompoundLiteralExpr => "CompoundLiteralExpr",
        CXCursorKind::InitListExpr => "InitListExpr",
        CXCursorKind::AddrLabelExpr => "AddrLabelExpr",
        CXCursorKind::StmtExpr => "StmtExpr",
        CXCursorKind::GenericSelectionExpr => "GenericSelectionExpr",
        CXCursorKind::GNUNullExpr => "GNUNullExpr",
        CXCursorKind::CXXStaticCastExpr => "CXXStaticCastExpr",
        CXCursorKind::CXXDynamicCastExpr => "CXXDynamicCastExpr",
        CXCursorKind::CXXReinterpretCastExpr => "CXXReinterpretCastExpr",
        CXCursorKind::CXXConstCastExpr => "CXXConstCastExpr",
        CXCursorKind::CXXFunctionalCastExpr => "CXXFunctionalCastExpr",
        CXCursorKind::CXXTypeidExpr => "CXXTypeidExpr",
        CXCursorKind::CXXBoolLiteralExpr => "CXXBoolLiteralExpr",
        CXCursorKind::CXXNullPtrLiteralExpr => "CXXNullPtrLiteralExpr",
        CXCursorKind::CXXThisExpr => "CXXThisExpr",
        CXCursorKind::CXXThrowExpr => "CXXThrowExpr",
        CXCursorKind::CXXNewExpr => "CXXNewExpr",
        CXCursorKind::CXXDeleteExpr => "CXXDeleteExpr",
        CXCursorKind::UnaryExpr => "UnaryExpr",
        CXCursorKind::ObjCStringLiteral => "ObjCStringLiteral",
        CXCursorKind::ObjCEncodeExpr => "ObjCEncodeExpr",
        CXCursorKind::ObjCSelectorExpr => "ObjCSelectorExpr",
        CXCursorKind::ObjCProtocolExpr => "ObjCProtocolExpr",
        CXCursorKind::ObjCBridgedCastExpr => "ObjCBridgedCastExpr",
        CXCursorKind::PackExpansionExpr => "PackExpansionExpr",
        CXCursorKind::SizeOfPackExpr => "SizeOfPackExpr",
        CXCursorKind::LambdaExpr => "LambdaExpr",
        CXCursorKind::ObjCBoolLiteralExpr => "ObjCBoolLiteralExpr",
        CXCursorKind::LabelStmt => "LabelStmt",
        CXCursorKind::CompoundStmt => "CompoundStmt",
        CXCursorKind::CaseStmt => "CaseStmt",
        CXCursorKind::DefaultStmt => "DefaultStmt",
        CXCursorKind::IfStmt => "IfStmt",
        CXCursorKind::SwitchStmt => "SwitchStmt",
        CXCursorKind::WhileStmt => "WhileStmt",
        CXCursorKind::DoStmt => "DoStmt",
        CXCursorKind::ForStmt => "ForStmt",
        CXCursorKind::GotoStmt => "GotoStmt",
        CXCursorKind::IndirectGotoStmt => "IndirectGotoStmt",
        CXCursorKind::ContinueStmt => "ContinueStmt",
        CXCursorKind::BreakStmt => "BreakStmt",
        CXCursorKind::ReturnStmt => "ReturnStmt",
        CXCursorKind::AsmStmt => "AsmStmt",
        CXCursorKind::ObjCAtTryStmt => "ObjCAtTryStmt",
        CXCursorKind::ObjCAtCatchStmt => "ObjCAtCatchStmt",
        CXCursorKind::ObjCAtFinallyStmt => "ObjCAtFinallyStmt",
        CXCursorKind::ObjCAtThrowStmt => "ObjCAtThrowStmt",
        CXCursorKind::ObjCAtSynchronizedStmt => "ObjCAtSynchronizedStmt",
        CXCursorKind::ObjCAutoreleasePoolStmt => "ObjCAutoreleasePoolStmt",
        CXCursorKind::ObjCForCollectionStmt => "ObjCForCollectionStmt",
        CXCursorKind::CXXCatchStmt => "CXXCatchStmt",
        CXCursorKind::CXXTryStmt => "CXXTryStmt",
        CXCursorKind::CXXForRangeStmt => "CXXForRangeStmt",
        CXCursorKind::SEHTryStmt => "SEHTryStmt",
        CXCursorKind::SEHExceptStmt => "SEHExceptStmt",
        CXCursorKind::SEHFinallyStmt => "SEHFinallyStmt",
        CXCursorKind::NullStmt => "NullStmt",
        CXCursorKind::DeclStmt => "DeclStmt",
        CXCursorKind::TranslationUnit => "TranslationUnit",
        CXCursorKind::IBActionAttr => "IBActionAttr",
        CXCursorKind::IBOutletAttr => "IBOutletAttr",
        CXCursorKind::IBOutletCollectionAttr => "IBOutletCollectionAttr",
        CXCursorKind::CXXFinalAttr => "CXXFinalAttr",
        CXCursorKind::CXXOverrideAttr => "CXXOverrideAttr",
        CXCursorKind::AnnotateAttr => "AnnotateAttr",
        CXCursorKind::AsmLabelAttr => "AsmLabelAttr",
        CXCursorKind::PreprocessingDirective => "PreprocessingDirective",
        CXCursorKind::MacroDefinition => "MacroDefinition",
        CXCursorKind::MacroExpansion => "MacroExpansion",
        CXCursorKind::InclusionDirective => "InclusionDirective",
        CXCursorKind::PackedAttr => "PackedAttr",

        _ => "?",
    }
}

pub fn type_to_str(x: CXTypeKind) -> &'static str {
    match x {
        CXTypeKind::Invalid => "Invalid",
        CXTypeKind::Unexposed => "Unexposed",
        CXTypeKind::Void => "Void",
        CXTypeKind::Bool => "Bool",
        CXTypeKind::Char_U =>  "Char_U",
        CXTypeKind::UChar => "UChar",
        CXTypeKind::Char16=> "Char16",
        CXTypeKind::Char32=> "Char32",
        CXTypeKind::UShort => "UShort",
        CXTypeKind::UInt => "UInt",
        CXTypeKind::ULong => "ULong",
        CXTypeKind::ULongLong => "ULongLong",
        CXTypeKind::UInt128=>"UInt128",
        CXTypeKind::Char_S => "Char_S",
        CXTypeKind::SChar => "SChar",
        CXTypeKind::WChar => "WChar",
        CXTypeKind::Short => "Short",
        CXTypeKind::Int => "Int",
        CXTypeKind::Long => "Long",
        CXTypeKind::LongLong => "LongLong",
        CXTypeKind::Int128=>"Int128",
        CXTypeKind::Float => "Float",
        CXTypeKind::Double => "Double",
        CXTypeKind::LongDouble => "LongDouble",
        CXTypeKind::NullPtr => "NullPtr",
        CXTypeKind::Overload => "Overload",
        CXTypeKind::Dependent => "Dependent",
        CXTypeKind::ObjCId => "ObjCId",
        CXTypeKind::ObjCClass => "ObjCClass",
        CXTypeKind::ObjCSel => "ObjCSel",
        CXTypeKind::Complex => "Complex",
        CXTypeKind::Pointer => "Pointer",
        CXTypeKind::BlockPointer => "BlockPointer",
        CXTypeKind::LValueReference => "LValueReference",
        CXTypeKind::RValueReference => "RValueReference",
        CXTypeKind::Record => "Record",
        CXTypeKind::Enum => "Enum",
        CXTypeKind::Typedef => "Typedef",
        CXTypeKind::ObjCInterface => "ObjCInterface",
        CXTypeKind::ObjCObjectPointer => "ObjCObjectPointer",
        CXTypeKind::FunctionNoProto => "FunctionNoProto",
        CXTypeKind::FunctionProto => "FunctionProto",
        CXTypeKind::ConstantArray => "ConstantArray",
        CXTypeKind::Vector => "Vector",
        CXTypeKind::IncompleteArray => "IncompleteArray",
        CXTypeKind::VariableArray => "VariableArray",
        CXTypeKind::DependentSizedArray => "DependentSizedArray",
        CXTypeKind::MemberPointer => "MemberPointer",
        #[cfg(not(feature="llvm_stable"))]
        CXTypeKind::Auto => "Auto",
        #[cfg(not(feature="llvm_stable"))]
        CXTypeKind::Elaborated => "Elaborated",
        _ => "?"
    }
}

// Debug
pub fn ast_dump(c: &Cursor, depth: isize)-> CXChildVisitResult {
    fn print_indent(depth: isize, s: &str) {
        let mut i = 0;
        while i < depth {
            print!("\t");
            i += 1;
        }
        println!("{}", s);
    }
    let ct = c.cur_type().kind();
    print_indent(depth, &format!("({} {} {}",
        kind_to_str(c.kind()),
        c.spelling(),
        type_to_str(ct))
    );
    c.visit(| s, _: &Cursor| {
        ast_dump(s, depth + 1)
    });
    print_indent(depth, ")");
    CXChildVisitResult::Continue
}
