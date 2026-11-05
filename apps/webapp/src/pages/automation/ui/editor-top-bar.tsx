import { Link } from '@tanstack/react-router'
import { RequirePermission } from '#/components/require-permission'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Switch } from '#/components/ui/switch'
import { Tabs, TabsList, TabsTrigger } from '#/components/ui/tabs'
import { buildOrgPath } from '#/modules/org-path'
import { workflowsPath } from '#/pages/automation/lib/automation-paths'

export type EditorView = 'editor' | 'executions'

export function EditorTopBar({
	organizationSlug,
	name,
	canRename,
	onNameChange,
	onNameBlur,
	tags,
	view,
	onViewChange,
	enabled,
	onEnabledChange,
	save,
}: {
	organizationSlug: string
	name: string
	canRename: boolean
	onNameChange: (name: string) => void
	onNameBlur: () => void
	tags: readonly string[]
	view: EditorView
	onViewChange: (view: EditorView) => void
	enabled: boolean
	onEnabledChange?: (enabled: boolean) => void
	save?: {
		dirty: boolean
		isSaving: boolean
		disabled?: boolean
		onSave: () => void
	}
}) {
	return (
		<header className="grid h-[52px] shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-4 border-border border-b bg-card px-4">
			<div className="flex min-w-0 items-center gap-2">
				<nav
					aria-label="Fil d’Ariane"
					className="flex min-w-0 items-center gap-1 text-base"
				>
					<Link
						to={buildOrgPath(organizationSlug, workflowsPath())}
						className="rounded-sm px-1 text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
					>
						Workflows
					</Link>
					<span className="text-muted-foreground" aria-hidden>
						/
					</span>
					<Input
						value={name}
						onChange={(event) => onNameChange(event.target.value)}
						onBlur={onNameBlur}
						onKeyDown={(event) => {
							if (event.key === 'Enter') event.currentTarget.blur()
						}}
						readOnly={!canRename}
						aria-label="Nom du workflow"
						size={Math.max(name.length, 10)}
						className="h-8 w-auto max-w-[36vw] border-transparent bg-transparent px-2 font-medium shadow-none hover:border-input md:text-base dark:bg-transparent"
					/>
				</nav>
				<ul
					className="hidden items-center gap-1 lg:flex"
					aria-label="Tags du workflow"
				>
					{tags.map((tag) => (
						<li key={tag}>
							<Badge variant="secondary">{tag}</Badge>
						</li>
					))}
				</ul>
			</div>
			<Tabs
				value={view}
				onValueChange={(value) =>
					onViewChange(value === 'executions' ? 'executions' : 'editor')
				}
			>
				<TabsList aria-label="Vue du workflow">
					<TabsTrigger value="editor" className="px-4">
						Éditeur
					</TabsTrigger>
					<TabsTrigger value="executions" className="px-4">
						Exécutions
					</TabsTrigger>
				</TabsList>
			</Tabs>
			<div className="flex items-center justify-end gap-3">
				<RequirePermission permission="MANAGE_AUTOMATION">
					<div className="flex items-center gap-2">
						<Label htmlFor="workflow-active">Actif</Label>
						<Switch
							id="workflow-active"
							checked={enabled}
							disabled={onEnabledChange === undefined}
							onCheckedChange={onEnabledChange}
						/>
					</div>
					{save ? (
						<Button
							size="sm"
							variant={save.dirty ? 'default' : 'outline'}
							disabled={save.isSaving || save.disabled}
							onClick={save.onSave}
						>
							{save.isSaving ? 'Enregistrement…' : 'Enregistrer'}
						</Button>
					) : null}
				</RequirePermission>
				{save ? (
					<output className="sr-only">
						{save.dirty ? 'Modifications non enregistrées' : 'Enregistré'}
					</output>
				) : null}
			</div>
		</header>
	)
}
