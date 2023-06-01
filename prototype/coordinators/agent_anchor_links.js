
export default {
    get_group ( group_id ) {
	const {
	    trace_evolution,
	}			= HDK;
	return trace_evolution( group_id ).pop().entry.asAppOption( EntryTypes.GroupEntry );
    },
    create_group ( group ) {
	const {
	    agent_info,
	    hash_entry,
	    create_entry,
	    create_link,
	}			= HDK;

	const agent		= agent_info().agent_initial_pubkey;
	const entry		= new EntryTypes.GroupEntry( group );

	const address		= hash_entry( entry );		// EntryHash
	const id		= create_entry( entry );	// ActionHash

	// Create anchors for each member
	for ( let pubkey of entry.authorities ) {
	    const anchor	= new EntryTypes.GroupMemberAnchorEntry([ id, pubkey ]);

	    const anchor_hash	= hash_entry( anchor );
	    create_entry( anchor );

	    console.log("Creating Group Member anchor (%s): %s", anchor_hash, anchor );
	    create_link( id, anchor_hash, LinkTypes.GroupMember, null );
	}

	create_link( agent, id, LinkTypes.Group, null );

	return id;
    },
    update_group ({ base, changes }) {
	const {
	    must_get_action,
	    must_get_entry,

	    hash_entry,
	    create_entry,
	    update_entry,

	    get_links,
	    create_link,

	    trace_origin,
	}			= HDK;

	const id		= trace_origin( base ).hash;
	console.log("Update Group (%s) off of base: %s", id, base );

	const action		= must_get_action( base ).action;
	const app_entry_bytes	= must_get_entry( action.entry_hash ).content;
	const prev_entry	= app_entry_bytes.toEntryType( EntryTypes.GroupEntry );
	const prev_authoritiess	= prev_entry.authoritiesSet();
	const entry		= Object.assign( prev_entry, changes );

	// console.log("Group authoritiess before:", prev_authoritiess );
	// console.log("Group authoritiess  after:", entry.authoritiesSet() );
	const authorities_diff	= prev_authoritiess.differences( entry.authoritiesSet() );
	// console.log("Group authorities diff:", authorities_diff );

	const address		= hash_entry( entry );		// EntryHash
	const action_hash	= update_entry( base, entry );	// ActionHash

	// Copy archive anchors from previous group revision unless the member was re-added
	let archive_links	= get_links( base, LinkTypes.GroupMemberArchived );
	for ( let link of archive_links ) {
	    create_link( action_hash, link.target, LinkTypes.GroupMemberArchived, link.tag )
	}

	// Move removed authorities' contributions to their archived anchor
	for ( let pubkey of authorities_diff.removed ) {
	    console.log("Removed Agent: %s", pubkey );
	    const anchor		= new EntryTypes.GroupMemberAnchorEntry([ id, pubkey ]);
	    const anchor_hash		= hash_entry( anchor );
	    const archive_anchor	= new EntryTypes.GroupMemberArchiveAnchorEntry([ id, pubkey, "archive" ]);
	    const archive_anchor_hash	= hash_entry( archive_anchor );

	    create_entry( archive_anchor );

	    console.log("Creating link to Group Member Archive for: %s", pubkey );
	    create_link( action_hash, archive_anchor_hash, LinkTypes.GroupMemberArchived, null );

	    let creates			= get_links( anchor_hash, LinkTypes.Subject );
	    let updates			= get_links( anchor_hash, LinkTypes.SubjectUpdate );

	    // console.log("Archived member contributions snapshot:", creates, updates );

	    creates.forEach( link => create_link( archive_anchor_hash, link.target, LinkTypes.Subject, link.tag ) );
	    updates.forEach( link => create_link( archive_anchor_hash, link.target, LinkTypes.SubjectUpdate, link.tag ) );
	}

	// Create anchors for new authorities
	for ( let pubkey of authorities_diff.added ) {
	    console.log("Added Agent: %s", pubkey );
	    const anchor	= new EntryTypes.GroupMemberAnchorEntry([ id, pubkey ]);

	    const anchor_hash	= hash_entry( anchor );
	    create_entry( anchor );

	    console.log("Creating Group Member anchor (%s): %s", anchor_hash, anchor );
	    create_link( action_hash, anchor_hash, LinkTypes.GroupMember, null );
	}

	// Create anchors for unchanged authorities
	for ( let pubkey of authorities_diff.intersection ) {
	    console.log("Unchanged Agent: %s", pubkey );
	    const anchor	= new EntryTypes.GroupMemberAnchorEntry([ id, pubkey ]);
	    const anchor_hash	= hash_entry( anchor );
	    console.log("Creating Group Member anchor (%s): %s", anchor_hash, anchor );
	    create_link( action_hash, anchor_hash, LinkTypes.GroupMember, null );
	}

	return action_hash;
    },
    get_group_subjects ({ id }) {
	const {
	    must_get_action,
	    must_get_entry,

	    trace_evolution,
	    get_links,

	    HoloHash,
	    ActionHash,
	}				= HDK;
	const latest_sa			= trace_evolution( id ).pop();
	const latest_base		= latest_sa.actionAddress();
	// console.log("%s", latest_sa );

	const group			= latest_sa.entry.asAppOption( EntryTypes.GroupEntry );
	// console.log("%s", group );

	const subjects			= {}; // latest_base => newest update
	const updates			= {}; // base => update

	function trace_updates ( base ) {
	    const link_map		= Object.assign( {}, updates );
	    // console.log("Tracing %s using known updates: %s", new HoloHash(base).toString(true), JSON.stringify(link_map, null, 4) );
	    const evolutions		= [ new ActionHash(base) ];

	    while ( String(base) in link_map ) {
		// console.log("Following update: %s => %s", new HoloHash(base).toString(true), link_map[ base ].toString(true) );
		new_base		= link_map[ base ];
		evolutions.push( new_base );

		delete link_map[ base ];
		base			= new_base;

		// console.log("Checking '%s' in link_map: %s", base, String(base) in link_map );
		// console.log("Checking '%s' in link_map keys: %s", base, Object.keys(link_map).includes( String(base) ) );
	    }

	    return evolutions;
	}

	const agent_anchors		= get_links( latest_base, LinkTypes.GroupMember );
	// console.log("Member anchor links:", agent_anchors );
	agent_anchors.forEach( link => {
	    const agent_anchor_entry	= must_get_entry( link.target ).content
		  .toEntryType( EntryTypes.GroupMemberAnchorEntry );
	    console.log("Group %s has archived member: %s", id.toString(true), agent_anchor_entry[1].toString(true) );

	    const links			= get_links( link.target, LinkTypes.Subject );
	    const ulinks		= get_links( link.target, LinkTypes.SubjectUpdate );
	    console.log("Archived members content: %s", {
		"Subject": links.length,
		"SubjectUpdate": ulinks.length,
	    });

	    console.log("Member %s subject:", agent_anchor_entry[1].toString(true), links.map( l => l.target.toString(true) ) );
	    links.forEach( slink => {
		const id		= slink.target;
		subjects[ id ]		= null;
	    });

	    console.log("Member %s subject updates:", agent_anchor_entry[1].toString(true), ulinks.map( l => l.target.toString(true) ) );
	    ulinks.forEach( ulink => {
		const [_, base]		= ulink.tag.split(":");
		const action_hash	= ulink.target;

		updates[ base ]		= action_hash;
	    });
	});

	const archived_agent_anchors	= get_links( latest_base, LinkTypes.GroupMemberArchived );
	archived_agent_anchors.forEach( link => {
	    const agent_anchor_entry	= must_get_entry( link.target ).content
		  .toEntryType( EntryTypes.GroupMemberArchiveAnchorEntry );
	    console.log("Group %s has archived member: %s", id.toString(true), agent_anchor_entry[1].toString(true) );

	    const links			= get_links( link.target, LinkTypes.Subject );
	    const ulinks		= get_links( link.target, LinkTypes.SubjectUpdate );
	    console.log("Archived members content: %s", {
		"Subject": links.length,
		"SubjectUpdate": ulinks.length,
	    });

	    console.log("Archived member %s subjects:", agent_anchor_entry[1].toString(true), links.map( l => l.target.toString(true) ) );
	    links.forEach( slink => {
		const id		= slink.target;
		subjects[ id ]		= null;
	    });

	    console.log("Archived member %s subject updates:", agent_anchor_entry[1].toString(true), ulinks.map( l => l.target.toString(true) ) );
	    ulinks.forEach( ulink => {
		const [_, base]		= ulink.tag.split(":");
		const action_hash	= ulink.target;

		updates[ base ]		= action_hash;
	    });
	});

	// console.log("Creates:", subjects );

	for ( let id in subjects ) {
	    console.log("Getting subject entry for ID: %s", id );
	    const evolutions		= trace_updates( id );
	    const latest_action		= evolutions.pop();
	    const saction		= must_get_action( latest_action );
	    const app_entry_bytes	= must_get_entry( saction.action.entry_hash ).content;

	    subjects[ id ]		= {
		"id": id,
		"action": saction.hash,
		"author": saction.action.author,
		"address": saction.action.entry_hash,
		"content": app_entry_bytes.toEntryType( EntryTypes.SubjectEntry ),
	    };
	}

	// console.log("Creates:", subjects );
	// console.log("Updates:", updates );

	return subjects;
    },
    get_subject ( subject_id ) {
	const {
	    trace_evolution,
	}			= HDK;
	return trace_evolution( subject_id ).pop().entry.asAppOption( EntryTypes.GroupEntry );
    },
    create_subject ( subject ) {
	const {
	    agent_info,
	    hash_entry,
	    create_entry,
	    create_link,
	}			= HDK;

	const agent		= agent_info().agent_initial_pubkey;
	const entry		= new EntryTypes.SubjectEntry( subject );
	const group_id		= entry.author[0];
	const group_rev		= entry.author[1];

	const address		= hash_entry( entry );		// EntryHash
	const id		= create_entry( entry );	// ActionHash

	const member_anchor	= new EntryTypes.GroupMemberAnchorEntry([ group_rev, agent ]);
	const anchor_hash	= hash_entry( member_anchor );

	console.log("Linking Group Member anchor (%s) to subject: %s", anchor_hash, id );
	// Group member anchor to Subject
	create_link( anchor_hash, id, LinkTypes.Subject, null );

	// Action author to Subject
	create_link( agent, id, LinkTypes.Subject, null );

	return id;
    },
    update_subject ({ base, changes }) {
	const {
	    agent_info,
	    must_get_action,
	    must_get_entry,

	    hash_entry,
	    update_entry,
	    create_link,

	    trace_origin,
	}			= HDK;

	const id		= trace_origin( base ).hash;
	const agent		= agent_info().agent_initial_pubkey;

	console.log("Update subject: %s", base );
	const action		= must_get_action( base ).action;
	const app_entry_bytes	= must_get_entry( action.entry_hash ).content;
	const prev_entry	= app_entry_bytes.toEntryType( EntryTypes.SubjectEntry );
	const group_id		= prev_entry.author[0];
	const group_rev		= prev_entry.author[1];
	const entry		= Object.assign( prev_entry, changes );

	const address		= hash_entry( entry );		// EntryHash
	const action_hash	= update_entry( base, entry );	// ActionHash

	const member_anchor	= new EntryTypes.GroupMemberAnchorEntry([ group_rev, agent ]);
	const anchor_hash	= hash_entry( member_anchor );

	// Group member anchor to [updated] Subject
	create_link( anchor_hash, action_hash, LinkTypes.SubjectUpdate, `${id}:${base}` );

	return action_hash;
    },
};
