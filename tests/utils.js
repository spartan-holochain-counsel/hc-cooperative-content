
import defaults				from 'defaults';
import { expect }			from 'chai';
import { faker }			from '@faker-js/faker';


export async function expect_reject ( cb, error, message ) {
    let failed				= false;
    try {
	await cb();
    } catch (err) {
	failed				= true;
	expect( () => { throw err }	).to.throw( error, message );
    }
    expect( failed			).to.be.true;
}

export function linearSuite ( name, setup_fn ) {
    describe( name, function () {
	beforeEach(function () {
	    let parent_suite		= this.currentTest.parent;
	    if ( parent_suite.tests.some(test => test.state === "failed") )
		this.skip();
	    if ( parent_suite.parent?.tests.some(test => test.state === "failed") )
		this.skip();
	});
	setup_fn.call( this );
    });
}

export function createGroupInput ( admins, ...members ) {
    return {
	"admins": admins,
	"members": [ ...members ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
	"metadata":		{},
    };
};

export function createContentInput ( group_id, group_rev, overrides = {} ) {
    return defaults( overrides, {
	"text":			faker.lorem.sentence(),
	"group_ref": {
	    "id":		group_id,
	    "rev":		group_rev,
	},

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
    });
};

export function createCommentInput ( group_id, group_rev, overrides = {} ) {
    return defaults( overrides, {
	"text":			faker.lorem.sentence(),
	"parent_comment":	null,
	"group_ref": {
	    "id":		group_id,
	    "rev":		group_rev,
	},
    });
};


export function delay ( n = 1 ) {
    return new Promise( f => setTimeout(f, n * 1_000) );
}


export default {
    expect_reject,
    linearSuite,
    createGroupInput,
    createContentInput,
    createCommentInput,
    delay,
};
