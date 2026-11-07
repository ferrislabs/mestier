import { Upload } from 'lucide-react'
import { type FormEvent, type ReactNode, useEffect, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { AddressKind } from '#/pages/customers/feature/mocks/account-extras'
import {
	type AddressForm,
	isAddressEmpty,
	validateAddressForm,
} from '#/pages/customers/lib/account-form'
import { ADDRESS_KIND_LABELS, ADDRESS_KINDS } from '#/pages/customers/types'
import { AddressFields } from '#/pages/customers/ui/address-fields'
import { FormErrors } from '#/pages/customers/ui/form-section'
import { useFormBinding } from '#/pages/customers/ui/use-form-binding'

export interface AddressDraft {
	label: string
	kind: AddressKind
	address: AddressForm
	photoKey: string
}

interface AddressFormDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	initial: AddressDraft
	trigger?: ReactNode
	isEditing: boolean
	isPending: boolean
	isUploadingPhoto: boolean
	error: string | null
	onUploadPhoto: (file: File) => Promise<string>
	onSubmit: (draft: AddressDraft) => Promise<void>
}

export function AddressFormDialog({
	open,
	onOpenChange,
	trigger,
	...form
}: AddressFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-lg">
				<AddressFormBody {...form} />
			</DialogContent>
		</Dialog>
	)
}

function AddressFormBody({
	title,
	initial,
	isEditing,
	isPending,
	isUploadingPhoto,
	error,
	onUploadPhoto,
	onSubmit,
}: Omit<AddressFormDialogProps, 'open' | 'onOpenChange' | 'trigger'>) {
	const [draft, setDraft] = useState<AddressDraft>(initial)
	const [previewUrl, setPreviewUrl] = useState<string | null>(null)
	const addressErrors = isAddressEmpty(draft.address)
		? {}
		: validateAddressForm(draft.address)
	const { bind, revealAll, formRef, baseId } = useFormBinding(addressErrors)
	const labelMissing = draft.label.trim() === ''

	useEffect(
		() => () => {
			if (previewUrl) URL.revokeObjectURL(previewUrl)
		},
		[previewUrl],
	)

	async function changePhoto(file: File) {
		setPreviewUrl(URL.createObjectURL(file))
		const key = await onUploadPhoto(file)
		setDraft((current) => ({ ...current, photoKey: key }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (labelMissing || Object.keys(addressErrors).length > 0) return
		void onSubmit(draft).catch(() => undefined)
	}

	return (
		<form
			ref={formRef}
			onSubmit={submit}
			noValidate
			className="flex min-h-0 flex-1 flex-col"
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>
					Le libellé suffit. L’adresse reste optionnelle selon le lieu : siège,
					chantier ou autre.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-3 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<FormField
					id={`${baseId}-label`}
					label="Libellé"
					required
					hint="Par exemple : Siège, Chantier Rezé."
				>
					<Input
						id={`${baseId}-label`}
						value={draft.label}
						onChange={(event) =>
							setDraft({ ...draft, label: event.target.value })
						}
						autoComplete="off"
					/>
				</FormField>
				<FormField id={`${baseId}-kind`} label="Type">
					<Select
						value={draft.kind}
						onValueChange={(value) => {
							const kind = ADDRESS_KINDS.find((item) => item === value)
							if (kind) setDraft({ ...draft, kind })
						}}
					>
						<SelectTrigger id={`${baseId}-kind`} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{ADDRESS_KINDS.map((kind) => (
								<SelectItem key={kind} value={kind}>
									{ADDRESS_KIND_LABELS[kind]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<AddressFields
					value={draft.address}
					onChange={(address) => setDraft({ ...draft, address })}
					bind={bind}
				/>
				<FormField
					id={`${baseId}-photo`}
					label="Photo du lieu"
					className="sm:col-span-2"
				>
					<div className="flex flex-col gap-3 sm:flex-row sm:items-center">
						<label
							htmlFor={`${baseId}-photo`}
							className="inline-flex h-8 cursor-pointer items-center justify-center gap-2 rounded-md border bg-background px-3 text-sm font-medium shadow-xs hover:bg-accent"
						>
							<Upload className="size-4" aria-hidden />
							{isUploadingPhoto ? 'Téléversement…' : 'Téléverser'}
						</label>
						<input
							id={`${baseId}-photo`}
							type="file"
							accept="image/*"
							className="sr-only"
							disabled={isUploadingPhoto}
							onChange={(event) => {
								const file = event.target.files?.[0]
								if (file) void changePhoto(file)
								event.currentTarget.value = ''
							}}
						/>
						{draft.photoKey ? (
							<span className="min-w-0 truncate font-mono text-xs text-muted-foreground">
								{draft.photoKey}
							</span>
						) : null}
						{previewUrl ? (
							<img
								src={previewUrl}
								alt="Aperçu du lieu"
								className="h-16 w-24 rounded-md border object-cover"
							/>
						) : null}
					</div>
				</FormField>
				<div className="sm:col-span-2">
					<FormErrors messages={error ? [error] : []} />
				</div>
			</div>
			<div className="flex justify-end gap-2 border-t px-6 py-3">
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button type="submit" disabled={isPending || isUploadingPhoto}>
					{isEditing ? 'Enregistrer' : 'Ajouter l’adresse'}
				</Button>
			</div>
		</form>
	)
}
