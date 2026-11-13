import { FolderInput, MoreHorizontal, PanelRight, Trash2 } from 'lucide-react'
import { Fragment, type ReactNode } from 'react'
import {
	Breadcrumb,
	BreadcrumbItem,
	BreadcrumbList,
	BreadcrumbPage,
	BreadcrumbSeparator,
} from '#/components/ui/breadcrumb'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Skeleton } from '#/components/ui/skeleton'

export interface DocumentCrumb {
	id: string
	name: string
}

interface DocumentPageProps {
	title: string
	canManage: boolean
	path: readonly DocumentCrumb[]
	indicator: ReactNode
	properties: ReactNode
	banner: ReactNode
	children: ReactNode
	onTitleChange: (title: string) => void
	onTitleCommit: () => void
	onOpenFolder: (folderId: string | null) => void
	onOpenInfo: () => void
	onMove: () => void
	onDelete: () => void
}

const CRUMB_CLASS =
	'max-w-48 truncate rounded px-1 py-0.5 outline-none transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50'

export function DocumentPage({
	title,
	canManage,
	path,
	indicator,
	properties,
	banner,
	children,
	onTitleChange,
	onTitleCommit,
	onOpenFolder,
	onOpenInfo,
	onMove,
	onDelete,
}: DocumentPageProps) {
	return (
		<div className="flex min-h-full flex-col bg-background">
			<header className="sticky top-0 z-20 flex h-12 shrink-0 items-center gap-2 bg-background/90 px-4 backdrop-blur">
				<Breadcrumb className="min-w-0 flex-1">
					<BreadcrumbList className="flex-nowrap gap-1 sm:gap-1">
						<BreadcrumbItem>
							<button
								type="button"
								className={CRUMB_CLASS}
								onClick={() => onOpenFolder(null)}
							>
								Drive
							</button>
						</BreadcrumbItem>
						{path.map((folder) => (
							<Fragment key={folder.id}>
								<BreadcrumbSeparator />
								<BreadcrumbItem className="min-w-0">
									<button
										type="button"
										className={CRUMB_CLASS}
										onClick={() => onOpenFolder(folder.id)}
									>
										{folder.name}
									</button>
								</BreadcrumbItem>
							</Fragment>
						))}
						<BreadcrumbSeparator />
						<BreadcrumbItem className="min-w-0">
							<BreadcrumbPage className="truncate px-1">
								{title.trim() === '' ? 'Sans titre' : title}
							</BreadcrumbPage>
						</BreadcrumbItem>
					</BreadcrumbList>
				</Breadcrumb>
				{indicator}
				<Button type="button" variant="ghost" size="sm" onClick={onOpenInfo}>
					<PanelRight aria-hidden />
					Infos
				</Button>
				{canManage ? (
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								aria-label="Actions du document"
							>
								<MoreHorizontal aria-hidden />
							</Button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end" className="min-w-44">
							<DropdownMenuItem onSelect={onMove}>
								<FolderInput aria-hidden />
								Déplacer
							</DropdownMenuItem>
							<DropdownMenuSeparator />
							<DropdownMenuItem variant="destructive" onSelect={onDelete}>
								<Trash2 aria-hidden />
								Supprimer
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				) : null}
			</header>
			<article className="document-surface pt-10 pb-40 sm:pt-16">
				<div className="grid gap-3 px-(--document-gutter) pb-6">
					<input
						type="text"
						aria-label="Titre du document"
						value={title}
						maxLength={200}
						readOnly={!canManage}
						placeholder="Sans titre"
						autoComplete="off"
						className="w-full bg-transparent font-semibold text-4xl leading-tight tracking-tight outline-none placeholder:text-muted-foreground/50"
						onChange={(event) => onTitleChange(event.target.value)}
						onBlur={onTitleCommit}
						onKeyDown={(event) => {
							if (event.key === 'Enter') event.currentTarget.blur()
						}}
					/>
					{properties}
					{banner}
				</div>
				{children}
			</article>
		</div>
	)
}

export function DocumentLoading() {
	return (
		<div className="flex min-h-full flex-col bg-background">
			<output className="sr-only">Chargement du document…</output>
			<div aria-hidden className="flex h-12 items-center gap-2 px-4">
				<Skeleton className="h-4 w-48" />
			</div>
			<div aria-hidden className="document-surface pt-10 sm:pt-16">
				<div className="grid gap-4 px-(--document-gutter)">
					<Skeleton className="h-10 w-2/3" />
					<Skeleton className="h-4 w-40" />
					<div className="grid gap-3 pt-6">
						<Skeleton className="h-4 w-full" />
						<Skeleton className="h-4 w-11/12" />
						<Skeleton className="h-4 w-4/5" />
						<Skeleton className="h-4 w-3/5" />
					</div>
				</div>
			</div>
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
