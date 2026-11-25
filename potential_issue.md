# Potential issues

## Edits during a paginated fetch can be missed (nov 25)
A client pulling pages (import, or GetAllYrsPages) isn't necessarily registered to receive love letters yet. If someone else's edit lands mid-fetch, the client may get neither the new page state (it already fetched that page) nor the letter.

Idea: make the client ready to receive letters before it starts pulling pages. Then an edit mid-fetch is either already in the pages or arrives as a letter afterward. Applying it twice should be harmless with yrs.

Open question: when does the server start rowing letters for a client? At login, or only after the first full sync? Need to check.

Pausing other requests during pagination was considered and rejected: the edit comes from another client, so the server would have to block everyone.
