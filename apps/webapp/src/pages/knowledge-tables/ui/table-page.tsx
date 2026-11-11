import { ChevronLeft, Loader2, Pencil } from 'lucide-react'
import { type ReactNode, useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'

interface TablePageProps {
	name: string
	description: string | null
	canManage: boolean
	isLoading: boolean
	error: string | null
	onBack: () => void
	onRename: (name: string) => void
	onEdit: () => void
	tabs: ReactNode
	toolbar: ReactNode
	children: ReactNode
}

function TableName({
	name,
	canManage,
	onRename,
}: {
	name: string
	canManage: boolean
	onRename: (name: string) => void
}) {
	const [draft, setDraft] = useState<string | null>(null)
	if (draft === null) {
		return canManage ? (
			<button
				type="button"
				title="Renommer la table"
				className="-mx-1 truncate rounded px-1 text-left font-semibold text-2xl tracking-tight outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
				onClick={() => setDraft(name)}
			>
				{name}
			</button>
		) : (
			<span className="truncate font-semibold text-2xl tracking-tight">
				{name}
			</span>
		)
	}
	const commit = () => {
		const next = draft.trim()
		setDraft(null)
		if (next !== '' && next !== name) onRename(next)
	}
	return (
		<Input
			autoFocus
			aria-label="Nom de la table"
			value={draft}
			maxLength={100}
			className="h-10 max-w-md font-semibold text-xl"
			onChange={(event) => setDraft(event.target.value)}
			onBlur={commit}
			onKeyDown={(event) => {
				if (event.key === 'Enter') {
					event.preventDefault()
					commit()
				} else if (event.key === 'Escape') {
					event.preventDefault()
					setDraft(null)
				}
			}}
		/>
	)
}

export function TablePage({
	name,
	description,
	canManage,
	isLoading,
	error,
	onBack,
	onRename,
	onEdit,
	tabs,
	toolbar,
	children,
}: TablePageProps) {
	return (
		<div className="flex w-full min-w-0 flex-col gap-3 px-6 py-6 lg:px-8">
			<header className="flex flex-wrap items-start justify-between gap-3">
				<div className="grid min-w-0 gap-1">
					<Button
						type="button"
						variant="link"
						size="sm"
						className="h-auto w-fit p-0 text-muted-foreground"
						onClick={onBack}
					>
						<ChevronLeft aria-hidden />
						Tables
					</Button>
					<h1 className="flex min-w-0 items-center gap-2">
						{isLoading ? (
							<span className="flex items-center gap-2 text-muted-foreground text-sm">
								<Loader2 aria-hidden className="size-4 animate-spin" />
								Chargement…
							</span>
						) : (
							<TableName
								key={name}
								name={name}
								canManage={canManage}
								onRename={onRename}
							/>
						)}
					</h1>
					{description ? (
						<p className="text-muted-foreground text-sm">{description}</p>
					) : null}
				</div>
				{canManage && !isLoading ? (
					<Button type="button" variant="outline" size="sm" onClick={onEdit}>
						<Pencil aria-hidden />
						Nom et description
					</Button>
				) : null}
			</header>
			{error ? (
				<div
					role="alert"
					className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-destructive text-sm"
				>
					{error}
				</div>
			) : null}
			{tabs}
			{toolbar}
			{children}
		</div>
	)
}
