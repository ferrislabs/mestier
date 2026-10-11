import { cn } from '#/lib/utils'
import { DOE_THEMES, type DoeTheme } from '#/pages/doe-style/lib/themes'

interface DoeThemePickerProps {
	activeId: string | null
	onApply: (theme: DoeTheme) => void
}

export function DoeThemePicker({ activeId, onApply }: DoeThemePickerProps) {
	return (
		<div className="flex flex-col gap-2">
			<p className="text-sm font-medium">Thèmes</p>
			<div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
				{DOE_THEMES.map((theme) => (
					<button
						key={theme.id}
						type="button"
						aria-pressed={theme.id === activeId}
						onClick={() => onApply(theme)}
						className={cn(
							'flex flex-col gap-2 rounded-lg border p-3 text-left transition hover:bg-muted/50',
							theme.id === activeId && 'border-primary ring-2 ring-primary/30',
						)}
					>
						<span className="flex h-3 overflow-hidden rounded-full">
							<span
								className="flex-1"
								style={{ backgroundColor: theme.look.primary }}
							/>
							<span
								className="flex-1"
								style={{ backgroundColor: theme.look.secondary }}
							/>
							<span
								className="flex-1 border"
								style={{ backgroundColor: theme.look.on_primary }}
							/>
						</span>
						<span className="text-sm font-medium">{theme.name}</span>
						<span className="text-xs text-muted-foreground">
							{theme.description}
						</span>
					</button>
				))}
			</div>
		</div>
	)
}
