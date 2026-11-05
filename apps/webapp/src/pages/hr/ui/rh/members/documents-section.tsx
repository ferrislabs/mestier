import { FileText, Paperclip, Trash2 } from 'lucide-react'
import { type ChangeEvent, useRef, useState } from 'react'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { formatDate } from '#/pages/hr/lib/rh/format'
import {
	addDocument,
	DOCUMENT_LABELS,
	removeDocument,
} from '#/pages/hr/lib/rh/members'
import type { DocumentKind, Member } from '#/pages/hr/lib/rh/types'
import { DetailCard } from '#/pages/hr/ui/rh/members/detail-card'
import { useRhUpdate } from '#/pages/hr/ui/rh/shared/rh-context'

export function DocumentsSection({ member }: { member: Member }) {
	const updateRh = useRhUpdate()
	const input = useRef<HTMLInputElement>(null)
	const [kind, setKind] = useState<DocumentKind>('other')

	function pick(event: ChangeEvent<HTMLInputElement>) {
		const file = event.target.files?.item(0)
		event.target.value = ''
		if (!file) return
		updateRh((state, context) =>
			addDocument(state, context, member.id, { name: file.name, kind }),
		)
	}

	return (
		<RubriqueLayout
			title="Documents"
			description="Pièces du dossier du membre."
			actions={
				<>
					<span aria-hidden className="text-[12px] text-muted-foreground">
						Type
					</span>
					<Select
						value={kind}
						onValueChange={(value) => setKind(value as DocumentKind)}
					>
						<SelectTrigger
							size="sm"
							className="w-44"
							aria-label="Type du document à ajouter"
						>
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{Object.entries(DOCUMENT_LABELS).map(([value, label]) => (
								<SelectItem key={value} value={value}>
									{label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
					<Button
						variant="outline"
						size="sm"
						onClick={() => input.current?.click()}
					>
						<Paperclip aria-hidden /> Ajouter un document
					</Button>
					<input
						ref={input}
						type="file"
						className="hidden"
						tabIndex={-1}
						aria-label="Choisir un fichier"
						onChange={pick}
					/>
				</>
			}
		>
			{member.documents.length === 0 ? (
				<div className="rounded-lg border border-dashed bg-card px-6 py-12 text-center">
					<p className="font-medium">Aucun document pour l’instant.</p>
					<p className="mt-1 text-[13px] text-muted-foreground">
						Ajoute un contrat, une pièce d’identité ou un diplôme avec le bouton
						ci-dessus.
					</p>
				</div>
			) : (
				<DetailCard title="Pièces du dossier">
					<ul className="-mx-1 grid divide-y">
						{member.documents.map((document) => (
							<li
								key={document.id}
								className="flex items-center gap-3 px-1 py-2.5 first:pt-0 last:pb-0"
							>
								<FileText
									className="size-4 shrink-0 text-muted-foreground"
									aria-hidden
								/>
								<span className="min-w-0 flex-1">
									<span className="block truncate text-[13px]">
										{document.name}
									</span>
									<span className="block text-[12px] text-muted-foreground">
										{DOCUMENT_LABELS[document.kind]} · ajouté le{' '}
										{formatDate(document.addedAt)}
									</span>
								</span>
								<Button
									variant="ghost"
									size="icon-sm"
									aria-label={`Supprimer ${document.name}`}
									onClick={() =>
										updateRh((state) =>
											removeDocument(state, member.id, document.id),
										)
									}
								>
									<Trash2 aria-hidden />
								</Button>
							</li>
						))}
					</ul>
				</DetailCard>
			)}
		</RubriqueLayout>
	)
}
