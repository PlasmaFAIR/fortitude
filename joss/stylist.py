from re import compile as recompile
from stylist.rule import TrailingWhitespace
from stylist.fortran import (
    FortranCharacterset,
    KindPattern,
    MissingImplicit,
    MissingIntent,
    MissingOnly,
    AutoCharArrayIntent,
    IntrinsicModule,
    LabelledDoExit,
    MissingPointerInit,
    NakedLiteral,
)
from stylist.style import Style


simple = Style(
    FortranCharacterset(),
    KindPattern(integer=recompile(r"i_.+"), real=recompile(r"r_.+")),
    MissingImplicit(),
    MissingIntent(),
    MissingOnly(),
    AutoCharArrayIntent(),
    IntrinsicModule(),
    LabelledDoExit(),
    MissingPointerInit(),
    NakedLiteral(),
)
