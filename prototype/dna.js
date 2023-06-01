
import {
    DNA,
}					from '@whi/holochain-prototyping';

import { PathEntry,
	 GroupMemberAnchorEntry,
	 GroupMemberArchiveAnchorEntry,
	 GroupEntry,
	 SubjectEntry }			from './entries.js';


export const dna = new DNA([
    {
	"EntryTypes": [
	    PathEntry,
	    GroupMemberAnchorEntry,
	    GroupMemberArchiveAnchorEntry,

	    GroupEntry,
	    SubjectEntry,
	],
	"LinkTypes": [
	    "Group",
	    "GroupMember",
	    "GroupMemberArchived",
	    "Subject",
	    "SubjectUpdate",
	],
	"validation": ( op ) => {
	    const {
		StoreEntry,
		StoreRecord,
		AppEntryBytes,
		OpEntry,
		CreateEntry,
		UpdateEntry,

		must_get_action,
		must_get_valid_record,
		trace_origin,
		heritage,
	    }			= HDI;
	    console.log("Validating %s from agent: %s", op.heritage(), op.author );

	    // if ( op instanceof StoreEntry ) {
	    // 	if ( op.entry instanceof AppEntryBytes ) {
	    // 	    console.log("Op has an AppEntry");
	    // 	}
	    // }

	    // console.log("EntryTypes:", EntryTypes );
	    const flat_op	= op.flattened( EntryTypes );
	    // console.log("%s", flat_op );

	    if ( flat_op instanceof OpEntry ) { // StoreEntry
		if ( flat_op instanceof CreateEntry ) {
		    console.log("Validate create entry");

		    if ( flat_op.app_entry instanceof EntryTypes.GroupEntry ) {
			console.log("Validate group");
		    }
		}

		if ( flat_op instanceof UpdateEntry ) {
		    console.log("Validate update entry");
		    if ( flat_op.app_entry instanceof EntryTypes.SubjectEntry ) {
			const subject		= flat_op.app_entry;
			const prev_record	= must_get_valid_record( flat_op.original_action_hash );
			const prev_subject	= prev_record.entry.asAppOption( EntryTypes.SubjectEntry );

			const group_record	= must_get_valid_record( prev_subject.author[1] );
			const group		= group_record.entry.asAppOption( EntryTypes.GroupEntry );

			const auths		= group.authorities.map( pk => String(pk) );

			console.log("Looking for %s in known auths:", op.author, group.authorities.map( pk => pk.toString(true) ) );
			// if ( !auths.includes( String(op.author) ) )
			//     throw new Error(`${op.author} does not have permission in author group (${prev_subject.author})`);
		    }

		    if ( flat_op.app_entry instanceof EntryTypes.GroupEntry ) {
			console.log("Validate group");
			const group		= flat_op.app_entry;
			const prev_record	= must_get_valid_record( flat_op.original_action_hash );
			const prev_group	= prev_record.entry.asAppOption( EntryTypes.GroupEntry );

			// Only an admin can update the auths
			const prev_admins	= new Set( prev_group.admins.map( pk => String(pk) ) );
			if ( !prev_admins.has( String(op.author) ) )
			    throw new Error(`${op.author} does not have permission; must be in group admin list: ${[...prev_admins.values()]}`);

			// Admin list must be the same
			const admins		= new Set( group.admins.map( pk => String(pk) ) );
			if ( prev_admins.size !== admins.size ||
			     ![...prev_admins].every( pk => admins.has( pk ) ) )
			    throw new Error(`Cannot change admin list without countersigning`);
		    }
		}
	    }
	},
    },
]);


export default {
    dna
};
