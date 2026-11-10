import { Plus } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import {
	formatNewsDate,
	newsState,
	newsStateLabel,
} from '#/pages/news-admin/lib/news-state'

interface NewsAdminListProps {
	news: ReadonlyArray<Schemas.NewsResponse>
	now: Date
	loading: boolean
	error: string | null
	onCreate: () => void
	onSelect: (news: Schemas.NewsResponse) => void
}

export function NewsAdminList({
	news,
	now,
	loading,
	error,
	onCreate,
	onSelect,
}: NewsAdminListProps) {
	return (
		<div className="mx-auto flex w-full max-w-5xl flex-col gap-4 p-6">
			<div className="flex items-center justify-between gap-3">
				<div>
					<h1 className="font-semibold text-xl">Back-office news</h1>
					<p className="text-muted-foreground text-sm">
						Les annonces affichées à tous les utilisateurs de Mestier.
					</p>
				</div>
				<Button type="button" onClick={onCreate}>
					<Plus />
					Nouvelle news
				</Button>
			</div>
			{error ? (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			) : null}
			<div className="overflow-hidden rounded-lg border bg-card">
				<Table>
					<TableHeader>
						<TableRow className="hover:bg-transparent">
							<TableHead className="pl-4">Titre</TableHead>
							<TableHead>État</TableHead>
							<TableHead className="max-md:hidden">Publication</TableHead>
							<TableHead className="pr-4 max-md:hidden">
								Dernière modification
							</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{news.map((item) => {
							const state = newsState(item.published_at, now)
							return (
								<TableRow
									key={item.id}
									className="cursor-pointer"
									tabIndex={0}
									onClick={() => onSelect(item)}
									onKeyDown={(event) => {
										if (event.key === 'Enter') onSelect(item)
									}}
								>
									<TableCell className="max-w-96 truncate py-3 pl-4 font-medium">
										{item.title}
									</TableCell>
									<TableCell>
										<Badge
											variant={state === 'published' ? 'default' : 'outline'}
										>
											{newsStateLabel(state)}
										</Badge>
									</TableCell>
									<TableCell className="max-md:hidden">
										{formatNewsDate(item.published_at)}
									</TableCell>
									<TableCell className="pr-4 max-md:hidden">
										{formatNewsDate(item.updated_at)}
									</TableCell>
								</TableRow>
							)
						})}
						{news.length === 0 ? (
							<TableRow className="hover:bg-transparent">
								<TableCell
									colSpan={4}
									className="py-8 text-center text-muted-foreground"
								>
									{loading ? 'Chargement…' : 'Aucune news pour le moment.'}
								</TableCell>
							</TableRow>
						) : null}
					</TableBody>
				</Table>
			</div>
		</div>
	)
}
