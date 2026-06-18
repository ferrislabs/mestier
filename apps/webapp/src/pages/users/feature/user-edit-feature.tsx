import { useForm } from '@tanstack/react-form'
import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, UserX } from 'lucide-react'
import { useRef } from 'react'
import { Button } from '#/components/ui/button'
import { useDirtyBaseline } from '#/hooks/use-dirty'
import {
	type User,
	useDeleteUser,
	useUpdateUser,
	useUsers,
} from '#/hooks/use-users'
import { type UserFormValues, userToForm } from '#/pages/users/types'
import { UserEditUI } from '#/pages/users/ui/user-edit-ui'

interface UserEditFeatureProps {
	userId: string
}

export function UserEditFeature({ userId }: UserEditFeatureProps) {
	const users = useUsers()
	const user = users.data?.data?.find((u) => u.id === userId) ?? null

	if (users.isLoading) {
		return <UserEditUI.Loading />
	}

	if (users.isError) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<AlertCircle className="size-6 text-destructive" />
				</div>
				<div>
					<p className="font-semibold">Impossible de charger l'utilisateur</p>
					<p className="text-sm text-muted-foreground">{users.error.message}</p>
				</div>
				<Button onClick={() => void users.refetch()} variant="outline">
					Réessayer
				</Button>
			</div>
		)
	}

	if (!user) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-lg border bg-card">
					<UserX className="size-6 text-muted-foreground" />
				</div>
				<div>
					<p className="font-semibold">Utilisateur introuvable</p>
					<p className="text-sm text-muted-foreground">
						Aucun compte ne correspond à cet identifiant.
					</p>
				</div>
				<Button asChild variant="outline">
					<Link to="/users">Retour aux utilisateurs</Link>
				</Button>
			</div>
		)
	}

	return <UserEditInner user={user} />
}

function UserEditInner({ user }: { user: User }) {
	const navigate = useNavigate()
	const updateUser = useUpdateUser()
	const deleteUser = useDeleteUser()
	const commitRef = useRef<(v: UserFormValues) => void>(() => {})

	const form = useForm({
		defaultValues: userToForm(user),
		onSubmit: async ({ value }) => {
			await updateUser.mutateAsync({
				path: { id: user.id },
				body: {
					email: value.email.trim() || null,
					username: value.username.trim() || null,
					name: value.name.trim() || null,
					enabled: value.enabled,
				},
			})
			commitRef.current(value)
		},
	})

	const handleDelete = async () => {
		await deleteUser.mutateAsync({ path: { id: user.id } })
		void navigate({ to: '/users' })
	}

	return (
		<form.Subscribe
			selector={(s) => ({ values: s.values, isSubmitting: s.isSubmitting })}
		>
			{({ values, isSubmitting }) => (
				<UserEditForm
					user={user}
					values={values}
					isSubmitting={isSubmitting || updateUser.isPending}
					isDeleting={deleteUser.isPending}
					form={form}
					commitRef={commitRef}
					onDelete={() => void handleDelete()}
				/>
			)}
		</form.Subscribe>
	)
}

interface UserEditFormProps {
	user: User
	values: UserFormValues
	isSubmitting: boolean
	isDeleting: boolean
	form: ReturnType<typeof useForm<UserFormValues>>
	commitRef: React.MutableRefObject<(v: UserFormValues) => void>
	onDelete: () => void
}

function UserEditForm({
	user,
	values,
	isSubmitting,
	isDeleting,
	form,
	commitRef,
	onDelete,
}: UserEditFormProps) {
	const baseline = userToForm(user)
	const {
		isDirty,
		commit,
		reset: resetBaseline,
	} = useDirtyBaseline(baseline, values)

	commitRef.current = commit

	return (
		<UserEditUI
			user={user}
			form={values}
			isDirty={isDirty}
			isSaving={isSubmitting}
			isDeleting={isDeleting}
			onChange={(patch) => {
				for (const key of Object.keys(patch) as (keyof UserFormValues)[]) {
					form.setFieldValue(key, patch[key] as never)
				}
			}}
			onReset={() => {
				form.reset()
				resetBaseline()
			}}
			onSave={() => form.handleSubmit()}
			onDelete={onDelete}
		/>
	)
}
