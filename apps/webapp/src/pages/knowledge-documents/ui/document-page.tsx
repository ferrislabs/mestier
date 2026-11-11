import { ArrowLeft, Info, Loader2, Trash2 } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'

interface DocumentPageProps {
	title: string
	canManage: boolean
	infoOpen: boolean
	indicator: ReactNode
	banner: ReactNode
	panel: ReactNode
	children: ReactNode
	onTitleChange: (title: string) => void
	onTitleCommit: () => void
	onToggleInfo: () => void
	onDelete: () => void
	onBack: () => void
}

export function DocumentPage({
	title,
	canManage,
	infoOpen,
	indicator,
	banner,
	panel,
	children,
	onTitleChange,
	onTitleCommit,
	onToggleInfo,
	onDelete,
	onBack,
}: DocumentPageProps) {
	return (
		<div className="mx-auto grid w-full max-w-6xl gap-4 px-6 py-8 lg:px-10">
			<header className="flex flex-wrap items-center gap-3">
				<Button type="button" variant="ghost" size="sm" onClick={onBack}>
					<ArrowLeft aria-hidden />
					Drive
				</Button>
				<span className="flex-1" />
				{indicator}
				<Button
					type="button"
					variant="outline"
					size="sm"
					aria-pressed={infoOpen}
					onClick={onToggleInfo}
				>
					<Info aria-hidden />
					Infos
				</Button>
				{canManage ? (
					<Button
						type="button"
						variant="ghost"
						size="icon-sm"
						aria-label="Supprimer le document"
						onClick={onDelete}
					>
						<Trash2 aria-hidden />
					</Button>
				) : null}
			</header>
			<Input
				aria-label="Titre du document"
				value={title}
				maxLength={200}
				readOnly={!canManage}
				placeholder="Sans titre"
				autoComplete="off"
				className="h-auto border-0 px-0 font-semibold text-3xl shadow-none focus-visible:ring-0"
				onChange={(event) => onTitleChange(event.target.value)}
				onBlur={onTitleCommit}
				onKeyDown={(event) => {
					if (event.key === 'Enter') event.currentTarget.blur()
				}}
			/>
			{banner}
			<div
				className={cn(
					'grid gap-6',
					infoOpen && 'lg:grid-cols-[minmax(0,1fr)_18rem]',
				)}
			>
				<div className="min-w-0">{children}</div>
				{infoOpen ? panel : null}
			</div>
		</div>
	)
}

export function DocumentLoading() {
	return (
		<div className="flex items-center justify-center gap-2 py-16 text-muted-foreground text-sm">
			<Loader2 aria-hidden className="size-4 animate-spin" />
			Chargement du document…
		</div>
	)
}

export function DocumentUnavailable({
	message,
	onBack,
}: {
	message: string
	onBack: () => void
}) {
	return (
		<div className="mx-auto grid w-full max-w-xl justify-items-center gap-4 px-6 py-16 text-center">
			<p role="alert" className="font-medium">
				{message}
			</p>
			<Button type="button" variant="outline" onClick={onBack}>
				Retour au Drive
			</Button>
		</div>
	)
}
