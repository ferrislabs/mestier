import type { ReactNode } from 'react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Label } from '#/components/ui/label'
import { PageHeader, PageShell } from '#/components/ui/surface'
import { Switch } from '#/components/ui/switch'
import { FormError } from '#/pages/cursus/participants/ui/field-input'

export interface CatalogPageLayout {
	description: string
	archived: boolean
	onArchivedChange: (archived: boolean) => void
}

export interface CatalogBlockProps {
	title: string
	page?: CatalogPageLayout
	addLabel: string
	canManage: boolean
	isLoading: boolean
	error: string | null
	isEmpty: boolean
	emptyLabel: string
	onAdd: () => void
	children: ReactNode
}

export function CatalogBlock({
	title,
	page,
	addLabel,
	canManage,
	isLoading,
	error,
	isEmpty,
	emptyLabel,
	onAdd,
	children,
}: CatalogBlockProps) {
	const list = (
		<>
			<FormError message={error} />
			{isLoading ? (
				<p className="text-sm text-muted-foreground">Chargement…</p>
			) : null}
			{!isLoading && isEmpty ? (
				<p className="text-sm text-muted-foreground">{emptyLabel}</p>
			) : null}
			<ul className="flex flex-col divide-y">{children}</ul>
		</>
	)

	if (page) {
		const switchId = `${title}-archived`
		return (
			<PageShell>
				<PageHeader
					title={title}
					description={page.description}
					actions={
						canManage ? (
							<Button type="button" onClick={onAdd}>
								{addLabel}
							</Button>
						) : null
					}
				/>
				<div className="flex items-center gap-2">
					<Switch
						id={switchId}
						checked={page.archived}
						onCheckedChange={page.onArchivedChange}
					/>
					<Label htmlFor={switchId}>Archivées</Label>
				</div>
				<section className="flex flex-col gap-3 rounded-lg border bg-card p-4">
					{list}
				</section>
			</PageShell>
		)
	}

	return (
		<section className="flex flex-col gap-3 rounded-lg border p-4">
			<div className="flex items-center justify-between gap-3">
				<h2 className="font-medium">{title}</h2>
				{canManage ? (
					<Button type="button" size="sm" onClick={onAdd}>
						{addLabel}
					</Button>
				) : null}
			</div>
			{list}
		</section>
	)
}

export interface CatalogRowProps {
	title: string
	subtitle?: string
	badge?: string
	archived: boolean
	canManage: boolean
	onEdit: () => void
	onArchive: () => void
}

export function CatalogRow({
	title,
	subtitle,
	badge,
	archived,
	canManage,
	onEdit,
	onArchive,
}: CatalogRowProps) {
	return (
		<li className="flex items-center justify-between gap-3 py-2">
			<div>
				<p className="flex items-center gap-2 font-medium">
					{title}
					{badge ? <Badge variant="outline">{badge}</Badge> : null}
					{archived ? <Badge variant="secondary">Archivé</Badge> : null}
				</p>
				{subtitle ? (
					<p className="text-sm text-muted-foreground">{subtitle}</p>
				) : null}
			</div>
			{canManage && !archived ? (
				<div className="flex gap-2">
					<Button
						type="button"
						variant="outline"
						size="sm"
						aria-label={`Modifier ${title}`}
						onClick={onEdit}
					>
						Modifier
					</Button>
					<Button
						type="button"
						variant="outline"
						size="sm"
						aria-label={`Archiver ${title}`}
						onClick={onArchive}
					>
						Archiver
					</Button>
				</div>
			) : null}
		</li>
	)
}

export interface CatalogDialogProps {
	open: boolean
	title: string
	description: string
	onOpenChange: (open: boolean) => void
	children: ReactNode
}

export function CatalogDialog({
	open,
	title,
	description,
	onOpenChange,
	children,
}: CatalogDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-lg">
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				{children}
			</DialogContent>
		</Dialog>
	)
}
