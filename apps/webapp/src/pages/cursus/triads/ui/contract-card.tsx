import { ExternalLink, Pencil, Users, XCircle } from 'lucide-react'
import { FileUploadButton } from '#/components/file-upload-button'
import { RichTextView } from '#/components/rich-text'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'
import {
	CONTRACT_KIND_LABELS,
	formatDate,
} from '#/pages/cursus/shared/lib/format'
import { ClosedBadge } from '#/pages/cursus/shared/ui/status-badge'
import type {
	Contract,
	ContractFileKind,
} from '#/pages/cursus/triads/feature/use-triads'
import { isClosed } from '#/pages/cursus/triads/lib/triad-form'

const FILE_LABELS: Record<ContractFileKind, string> = {
	mission_sheet: 'Fiche mission',
}

const FILE_KINDS: ContractFileKind[] = ['mission_sheet']

function hasFile(contract: Contract, kind: ContractFileKind): boolean {
	return kind === 'mission_sheet' && contract.has_mission_sheet_file
}

interface ContractCardProps {
	contract: Contract
	canManage: boolean
	isUploading: boolean
	onEdit: () => void
	onTutors: () => void
	onClose: () => void
	onOpenFile: (kind: ContractFileKind) => void
	onUploadFile: (kind: ContractFileKind, file: File) => void
}

export function ContractCard({
	contract,
	canManage,
	isUploading,
	onEdit,
	onTutors,
	onClose,
	onOpenFile,
	onUploadFile,
}: ContractCardProps) {
	const closed = isClosed(contract)
	const editable = canManage && !closed

	return (
		<article className="grid gap-3 rounded-lg border bg-card p-4">
			<div className="flex flex-wrap items-start justify-between gap-3">
				<div>
					<h3 className="font-medium">{contract.host_company_name}</h3>
					<p className="text-[13px] text-muted-foreground">
						{formatDate(contract.start_date)} → {formatDate(contract.end_date)}
						{contract.number ? ` · n° ${contract.number}` : ''}
					</p>
				</div>
				<div className="flex items-center gap-2">
					<Badge variant="outline">{CONTRACT_KIND_LABELS[contract.kind]}</Badge>
					<ClosedBadge closed={closed} />
				</div>
			</div>

			<dl className="grid gap-1 text-[13px] sm:grid-cols-2">
				<div>
					<dt className="text-muted-foreground">Durée</dt>
					<dd>{contract.duration_months} mois</dd>
				</div>
				<div>
					<dt className="text-muted-foreground">Tuteurs entreprise</dt>
					<dd>
						{contract.tutors.length === 0 ? (
							'—'
						) : (
							<ul className="grid gap-1">
								{contract.tutors.map((tutor) => (
									<li
										key={tutor.company_tutor_id}
										className="flex items-center gap-2"
									>
										{tutor.name}
										{tutor.is_main ? <Badge>Principal</Badge> : null}
									</li>
								))}
							</ul>
						)}
					</dd>
				</div>
			</dl>

			{contract.comment ? (
				<div className="grid gap-1 text-[13px]">
					<p className="text-muted-foreground">Commentaire</p>
					<RichTextView markdown={contract.comment} />
				</div>
			) : null}

			{closed ? (
				<div className="rounded-md bg-muted px-3 py-2 text-[13px]">
					<p>
						Clôturé le {formatDate(contract.closed_at)}
						{contract.closure_reason_label
							? ` · ${contract.closure_reason_label}`
							: ''}
					</p>
					{contract.closure_note ? (
						<p className="text-muted-foreground">{contract.closure_note}</p>
					) : null}
				</div>
			) : null}

			<ul className="grid gap-2">
				{FILE_KINDS.map((kind) => (
					<li
						key={kind}
						className="flex flex-wrap items-center justify-between gap-2 text-[13px]"
					>
						<span>
							{FILE_LABELS[kind]}
							{hasFile(contract, kind) ? '' : ' — aucun fichier'}
						</span>
						<div className="flex gap-2">
							{hasFile(contract, kind) ? (
								<Button
									type="button"
									variant="outline"
									size="sm"
									aria-label={`Ouvrir ${FILE_LABELS[kind]}`}
									onClick={() => onOpenFile(kind)}
								>
									<ExternalLink />
									Ouvrir
								</Button>
							) : null}
							{editable ? (
								<FileUploadButton
									accept="application/pdf,image/*"
									isUploading={isUploading}
									onFiles={(files) => {
										const file = files[0]
										if (file) onUploadFile(kind, file)
									}}
								>
									{hasFile(contract, kind) ? 'Remplacer' : 'Téléverser'}
								</FileUploadButton>
							) : null}
						</div>
					</li>
				))}
			</ul>

			{editable ? (
				<div className="flex flex-wrap gap-2 border-t pt-3">
					<Button type="button" variant="outline" size="sm" onClick={onEdit}>
						<Pencil />
						Modifier
					</Button>
					<Button type="button" variant="outline" size="sm" onClick={onTutors}>
						<Users />
						Tuteurs
					</Button>
					<Button type="button" variant="outline" size="sm" onClick={onClose}>
						<XCircle />
						Clôturer le contrat
					</Button>
				</div>
			) : null}
		</article>
	)
}
