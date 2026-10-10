import { X } from 'lucide-react'
import { FileUploadButton } from '#/components/file-upload-button'
import { TextField } from '#/components/reference-table'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import {
	type DoeForm,
	PARTY_LABELS,
	type PartyForm,
	type PartyRole,
} from '#/pages/doe/types'

interface DoeInfoTabProps {
	form: DoeForm
	fileUrls: Record<string, string | undefined>
	isUploadingLogo: PartyRole | null
	onFieldChange: (patch: Partial<Omit<DoeForm, 'parties' | 'sections'>>) => void
	onPartyChange: (role: PartyRole, patch: Partial<PartyForm>) => void
	onLogoUpload: (role: PartyRole, file: File) => void
}

export function DoeInfoTab({
	form,
	fileUrls,
	isUploadingLogo,
	onFieldChange,
	onPartyChange,
	onLogoUpload,
}: DoeInfoTabProps) {
	const revisionLength = form.revision.trim().length
	const revisionInvalid = revisionLength < 1 || revisionLength > 8

	return (
		<div className="flex flex-col gap-6">
			<SectionCard>
				<SectionHeader
					title="Informations"
					description="Le chantier tel qu'il apparaît sur la page de garde."
				/>
				<div className="grid gap-4 p-5 md:grid-cols-2">
					<TextField
						label="Titre du chantier"
						value={form.title}
						onChange={(title) => onFieldChange({ title })}
					/>
					<TextField
						label="Lieu d'intervention"
						value={form.place}
						onChange={(place) => onFieldChange({ place })}
					/>
					<TextField
						label="N° de lot"
						value={form.lot}
						onChange={(lot) => onFieldChange({ lot })}
					/>
					<div className="flex flex-col gap-2">
						<Label htmlFor="doe-issued-on">Date</Label>
						<Input
							id="doe-issued-on"
							type="date"
							value={form.issuedOn}
							onChange={(event) =>
								onFieldChange({ issuedOn: event.target.value })
							}
						/>
					</div>
					<TextField
						label="Rédacteur"
						value={form.author}
						onChange={(author) => onFieldChange({ author })}
					/>
					<div className="flex flex-col gap-2">
						<Label htmlFor="doe-revision">Indice</Label>
						<Input
							id="doe-revision"
							value={form.revision}
							maxLength={8}
							aria-invalid={revisionInvalid}
							onChange={(event) =>
								onFieldChange({ revision: event.target.value })
							}
						/>
						<p className="text-xs text-muted-foreground">1 à 8 caractères.</p>
					</div>
				</div>
			</SectionCard>

			<SectionCard>
				<SectionHeader
					title="Intervenants"
					description="Un intervenant sans nom n'est pas imprimé dans le DOE."
				/>
				<div className="grid gap-4 p-5 lg:grid-cols-3">
					{form.parties.map((party) => (
						<PartyCard
							key={party.role}
							party={party}
							logoUrl={party.logoKey ? fileUrls[party.logoKey] : undefined}
							isUploadingLogo={isUploadingLogo === party.role}
							onChange={(patch) => onPartyChange(party.role, patch)}
							onLogoUpload={(file) => onLogoUpload(party.role, file)}
						/>
					))}
				</div>
			</SectionCard>
		</div>
	)
}

interface PartyCardProps {
	party: PartyForm
	logoUrl: string | undefined
	isUploadingLogo: boolean
	onChange: (patch: Partial<PartyForm>) => void
	onLogoUpload: (file: File) => void
}

function PartyCard({
	party,
	logoUrl,
	isUploadingLogo,
	onChange,
	onLogoUpload,
}: PartyCardProps) {
	const label = PARTY_LABELS[party.role]
	const prefix = `party-${party.role}`

	return (
		<fieldset className="flex flex-col gap-3 rounded-lg border p-4">
			<legend className="px-1 text-sm font-medium">{label}</legend>
			<TextField
				label={`Nom (${label})`}
				value={party.name}
				onChange={(name) => onChange({ name })}
			/>
			<div className="flex flex-col gap-2">
				<Label htmlFor={`${prefix}-address`}>Adresse ({label})</Label>
				<Textarea
					id={`${prefix}-address`}
					value={party.address}
					rows={3}
					onChange={(event) => onChange({ address: event.target.value })}
				/>
			</div>
			<TextField
				label={`Téléphone (${label})`}
				value={party.phone}
				onChange={(phone) => onChange({ phone })}
			/>
			<TextField
				label={`Email (${label})`}
				type="email"
				value={party.email}
				onChange={(email) => onChange({ email })}
			/>
			<div className="flex items-center gap-3">
				{party.logoKey ? (
					<div className="flex items-center gap-2">
						{logoUrl ? (
							<img
								src={logoUrl}
								alt={`Logo ${label}`}
								className="h-10 max-w-24 rounded border object-contain"
							/>
						) : (
							<span className="text-xs text-muted-foreground">Logo envoyé</span>
						)}
						<RequirePermission permission="MANAGE_PLANNING">
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								onClick={() => onChange({ logoKey: null })}
							>
								<X />
								<span className="sr-only">Retirer le logo ({label})</span>
							</Button>
						</RequirePermission>
					</div>
				) : (
					<span className="text-xs text-muted-foreground">Aucun logo</span>
				)}
				<RequirePermission permission="MANAGE_PLANNING">
					<FileUploadButton
						accept="image/*"
						isUploading={isUploadingLogo}
						onFiles={(files) => {
							const file = files[0]
							if (file) onLogoUpload(file)
						}}
					>
						Logo
					</FileUploadButton>
				</RequirePermission>
			</div>
		</fieldset>
	)
}
