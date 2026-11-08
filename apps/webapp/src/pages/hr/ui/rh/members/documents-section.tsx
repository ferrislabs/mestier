import { FileText, Paperclip, Trash2 } from 'lucide-react'
import { type ChangeEvent, useRef, useState } from 'react'
import { RequirePermission } from '#/components/require-permission'
import { RubriqueLayout } from '#/components/rubrique-layout'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { useFileUrls } from '#/hooks/use-file-url'
import { mutationErrorMessage } from '#/lib/api-error'
import { formatDate } from '#/pages/hr/lib/rh/format'
import { DOCUMENT_LABELS } from '#/pages/hr/lib/rh/members'
import type { DocumentKind, Member } from '#/pages/hr/lib/rh/types'
import { DetailCard } from '#/pages/hr/ui/rh/members/detail-card'
import { useRhDocumentActions } from '#/pages/hr/ui/rh/shared/rh-context'

export function DocumentsSection({ member }: { member: Member }) {
	const actions = useRhDocumentActions()
	const input = useRef<HTMLInputElement>(null)
	const [kind, setKind] = useState<DocumentKind>('other')
	const [pending, setPending] = useState(false)
	const [error, setError] = useState<string | null>(null)
	const links = useFileUrls(
		member.documents.map((document) => document.fileKey),
	)

	async function run(action: () => Promise<void>) {
		setPending(true)
		setError(null)
		try {
			await action()
		} catch (failure) {
			setError(mutationErrorMessage(failure))
		} finally {
			setPending(false)
		}
	}

	function pick(event: ChangeEvent<HTMLInputElement>) {
		const file = event.target.files?.item(0)
		event.target.value = ''
		if (!file) return
		void run(() => actions.add(member.id, file, kind))
	}

	return (
		<RubriqueLayout
			title="Documents"
			description="Pièces du dossier du membre."
			actions={
				<RequirePermission permission="MANAGE_MEMBERS">
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
						disabled={pending}
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
				</RequirePermission>
			}
		>
			{error && (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			)}
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
						{member.documents.map((document, index) => {
							const url = links[index]?.url
							return (
								<li
									key={document.id}
									className="flex items-center gap-3 px-1 py-2.5 first:pt-0 last:pb-0"
								>
									<FileText
										className="size-4 shrink-0 text-muted-foreground"
										aria-hidden
									/>
									<span className="min-w-0 flex-1">
										{url ? (
											<a
												href={url}
												target="_blank"
												rel="noreferrer"
												className="block truncate text-[13px] hover:underline"
											>
												{document.name}
											</a>
										) : (
											<span className="block truncate text-[13px]">
												{document.name}
											</span>
										)}
										<span className="block text-[12px] text-muted-foreground">
											{DOCUMENT_LABELS[document.kind]} · ajouté le{' '}
											{formatDate(document.addedAt)}
										</span>
									</span>
									<RequirePermission permission="MANAGE_MEMBERS">
										<Button
											variant="ghost"
											size="icon-sm"
											disabled={pending}
											aria-label={`Supprimer ${document.name}`}
											onClick={() =>
												void run(() => actions.remove(member.id, document.id))
											}
										>
											<Trash2 aria-hidden />
										</Button>
									</RequirePermission>
								</li>
							)
						})}
					</ul>
				</DetailCard>
			)}
		</RubriqueLayout>
	)
}
