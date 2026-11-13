/* The controls of a block stay out of the reading: they appear when the
   block is hovered or holds the focus, and always on touch screens. */
export const BLOCK_TOOLBAR =
	'z-10 flex items-center gap-0.5 rounded-md border bg-popover p-0.5 text-popover-foreground shadow-sm opacity-0 transition-opacity group-hover/block:opacity-100 group-focus-within/block:opacity-100 pointer-coarse:opacity-100'

export const BLOCK_TOOL_BUTTON =
	'inline-flex h-7 items-center gap-1.5 rounded px-2 text-muted-foreground text-xs outline-none transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 [&_svg]:size-3.5'
