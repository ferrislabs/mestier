import { cn } from '#/lib/utils'
import { DOE_THEMES, type DoeTheme } from '#/pages/doe-style/lib/themes'

interface DoeThemePickerProps {
	activeId: string | null
	onApply: (theme: DoeTheme) => void
}

export function DoeThemePicker({ activeId, onApply }: DoeThemePickerProps) {
	return (
		<fieldset className="grid min-w-0 gap-2">
			<legend className="font-medium text-sm">Thèmes prêts à l’emploi</legend>
			<p className="text-[12px] text-muted-foreground">
				Un thème règle les couleurs, les polices et la couverture. Le logo,
				l’en-tête, le pied de page et les libellés sont conservés.
			</p>
			<ul className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
				{DOE_THEMES.map((theme) => {
					const active = theme.id === activeId
					return (
						<li key={theme.id} className="flex">
							<button
								type="button"
								aria-pressed={active}
								onClick={() => onApply(theme)}
								className={cn(
									'flex w-full flex-col gap-1.5 rounded-lg border bg-background p-3 text-left outline-none transition-colors hover:bg-muted/50 focus-visible:ring-[3px] focus-visible:ring-ring/50',
									active && 'border-primary ring-2 ring-primary/30',
								)}
							>
								<span
									aria-hidden
									className="flex h-3 overflow-hidden rounded-full border"
								>
									<span
										className="flex-1"
										style={{ backgroundColor: theme.look.primary }}
									/>
									<span
										className="flex-1"
										style={{ backgroundColor: theme.look.secondary }}
									/>
									<span
										className="flex-1"
										style={{ backgroundColor: theme.look.on_primary }}
									/>
								</span>
								<span className="font-medium text-sm">{theme.name}</span>
								<span className="text-[12px] text-muted-foreground">
									{theme.description}
								</span>
							</button>
						</li>
					)
				})}
			</ul>
		</fieldset>
	)
}
