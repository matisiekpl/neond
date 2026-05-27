# Accounts & organizations

## The first user

Open neond for the first time and you get a setup screen: pick an email and
password, submit. That account becomes the instance administrator.

## Signup is closed after that

Once the first user exists, signup is off. There is no public registration
form, no invite link, no way to self-create an account. New users only come
from an administrator.

## Adding more people

From the **Users** screen, an admin clicks **Add user**, fills in a name,
email, and initial password, and shares the credentials with the new user.
They can sign in right away.

Admins can also reset passwords, promote or demote other users, and delete
accounts. You can't demote or delete yourself.

## Organizations

An organization holds projects, branches, endpoints, and snapshots. You only
see an organization's contents if you're a member of it.

- **Create.** Any user can create one and becomes its first member.
- **Add members.** From the **Members** tab, enter the email of an existing
  neond user. The account has to exist first — there are no invites.
- **Remove members.** Any member can remove any other member, except the
  last one.
- **Delete.** Removes the organization and everything inside it.
  Irreversible.

There are no per-organization roles. Every member can do everything in the
organization, including adding members or deleting it. The instance admin
flag is separate and doesn't give access to organizations you haven't been
added to.