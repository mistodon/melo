" Vim syntax file
" Language: Melo
" Maintainer: ***Realname*** ***Lastname***
" Latest Revision: 2018

if exists("b:current_syntax")
  finish
endif


syn match meloError '.'
syn match meloWhitespace '\s'
syn match meloDelim '[{},]'
syn match meloNote '[a-gA-G][_\#=]*[,\']*'
syn match meloSymbol '[\-x.%]'
syn match meloLength '\d\+'
syn match meloBarline '[|]'
syn match meloName '[a-zA-Z0-9_][a-zA-Z0-9_ ]*'
syn match meloName '".*"'
syn match meloValue '[a-zA-Z0-9_][a-zA-Z0-9_ ]*'
syn match meloKey '[a-zA-Z\#=_,':]\?[a-zA-Z0-9\#=_,':]*:'
syn match meloComment '//.*$'

syn keyword meloKeyword piece voice play section part drums nextgroup=meloName skipwhite

syn region meloStave start="|" end="\n" fold transparent contains=meloNote,meloLength,meloBarline,meloSymbol,meloComment,meloError,meloWhitespace,meloDelim

let b:current_syntax = "melo"

hi def link meloKeyword Keyword
hi def link meloKey PreProc
hi def link meloValue Constant
hi def link meloName Type
hi def link meloNote Identifier
hi def link meloLength Constant
hi def link meloBarline PreProc
hi def link meloSymbol Identifier
hi def link meloComment Comment
hi def link meloError Error
