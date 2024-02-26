#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

import os, sys
import random
import argparse
import logging
from datetime import datetime, timedelta, timezone
import yaml
from github import Github, Auth

# Cache GitHub API requests aggressively:
from requests_cache import NEVER_EXPIRE, DO_NOT_CACHE, get_cache, install_cache
install_cache(
    cache_control=True,
    urls_expire_after={
        '*.github.com': NEVER_EXPIRE,
        '*': DO_NOT_CACHE,
    },
)


def ignore_prs_filter(config, prs):
    if "ignored_label" in config:
        return filter(
            lambda pr: not any(map(
                lambda l: l.name == config["ignored_label"],
                pr.get_labels()
            )),
            prs
        )
    else:
        return prs

def verbose_pr_stream(prs, log):
    def verbose_pr_stream_log(pr, log):
        log.debug(f"Processing PR #{pr.number} (\"{pr.title}\")")
        return pr
    return map(lambda pr: verbose_pr_stream_log(pr, log), prs)

# Assign maintainers to stale PRs when they haven't seen any review /
# reviewer activity after a given amount of time:
def task_stale_pr_assign(config, task_config, gh, repo, rand, log, dry_run):
    # Get the list of open PRs:
    prs = verbose_pr_stream(repo.get_pulls(state="open"), log)

    # Filter out PRs that are marked as ignored by this tool:
    prs = ignore_prs_filter(config, prs)

    # Filter out PRs that are assigned to one or more users:
    prs = filter(lambda pr: len(pr.assignees) == 0, prs)

    # Filter out PRs which have received reviews that are not dismissed
    # (optionally filted by a designated group of people, if the config is not
    # an empty list):
    no_reviews_cond = task_config.get("no_reviews_by", None)
    if no_reviews_cond is not None:
        prs = filter(
            lambda pr: not any(map(
                lambda review: (
                    # Only keep PRs that do not have any review where the
                    # reviewer is in the `no_reviews_cond` list, ...
                    review.user.login in no_reviews_cond \
                    # ... not counting dismissed reviews:
                    and review.state != "DISMISSED" \
                    # ... and not comment reviews (won't be dismissed):
                    and review.state != "COMMENTED"
                ),
                pr.get_reviews(),
            )),
            prs
        )

    # Filter our PRs that have seen a comment be updated in the last
    # task_config["staleness_time"] seconds:
    if task_config.get("staleness_time", None) is not None:
        comments_since = datetime.now(timezone.utc) \
            - timedelta(seconds=task_config["staleness_time"])

        prs = filter(
            lambda pr: (
                (
                    # Keep PRs that do _not_ have at least one review comment or
                    # at least one issue comment since `comments_since`,
                    pr.get_review_comments(since=comments_since).totalCount == 0 and \
                    pr.as_issue().get_comments(since=comments_since).totalCount == 0
                ) and (
                    # ... except if the PR is less than `staleness_time` old:
                    pr.created_at < comments_since
                )
            ),
            prs
        )

    # Now, add an assignee to all remaining PRs randomly:
    assignee_cnt = task_config.get("assignee_cnt", 1)
    for pr in prs:
        assignees = list(map(
            lambda login: gh.get_user(login),
            rand.sample(
                list(filter(
                    # Avoid assigning the PR creator:
                    lambda login: pr.user.login != login,
                    task_config["assignee_candidates"])),
                assignee_cnt
            )
        ))

        log.info((
            "Would assign user(s) {} to PR #{} (\"{}\")"
            if dry_run else
            "Assigning user(s) {} to PR #{} (\"{}\")"
        ).format(
            ", ".join(map(lambda a: a.login, assignees)),
            pr.number,
            pr.title,
        ))

        if not dry_run:
            pr.add_to_assignees(*assignees)


def cmd_maint_nightly(config, log, dry_run, gh_token = None):
    rand = random.SystemRandom()

    # Instantiate the GitHub client library:
    if gh_token is not None:
        auth_args = { "auth": Auth.Token(gh_token) }
    else:
        log.warning("Running without GitHub auth token.")
        auth_args = {}

    gh = Github(**auth_args)

    repo = gh.get_repo("{}/{}".format(
        config["repo"]["owner"],
        config["repo"]["name"]))

    # Perform the various maintenance tasks
    task_handlers = {
        "stale_pr_assign": task_stale_pr_assign,
    }

    for task in config["tasks"]:
        if task["type"] not in task_handlers:
            log.error("Unknown task type \"{}\", skipping!".format(task["type"]))
            continue

        log.info("Running task \"{}\" (type \"{}\")...".format(
            task.get("label", ""), task["type"]))
        log.debug(f"Starting task with rate limits: {str(gh.get_rate_limit())}")

        handler = task_handlers[task["type"]]
        handler(
            config = config,
            task_config = task,
            gh = gh,
            repo = repo,
            rand = rand,
            log = log,
            dry_run = dry_run,
        )

    log.debug(f"Finished all tasks with rate limits: {str(gh.get_rate_limit())}")

def main():
    parser = argparse.ArgumentParser(prog = "tockbot")

    # Global options:
    parser.add_argument("-n", "--dry-run", action="store_true")
    parser.add_argument("-v", "--verbose", action="store_true")

    # Subcommands:
    subparsers = parser.add_subparsers(dest="subcommand", required=True)

    # Nightly project maintenance command:
    maint_nightly_parser = subparsers.add_parser("maint-nightly")
    maint_nightly_parser.add_argument(
        "-c", "--config", required=True,
        help="YAML configuration for nightly maintenance job")

    args = parser.parse_args()

    # Initialize the logging facility:
    ch = logging.StreamHandler()
    fmt = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    ch.setFormatter(fmt)
    log = logging.getLogger('tockbot')
    log.addHandler(ch)
    if args.verbose:
        log.setLevel(logging.DEBUG)
    else:
        log.setLevel(logging.INFO)

    # Load the YAML configuration for commands that require it:
    if args.subcommand in ["maint-nightly"]:
        with open(args.config, "r") as f:
            config = yaml.safe_load(f)

    # Check if we're being passed a GitHub access token in an environment var:
    gh_token = os.environ.get("GITHUB_TOKEN", None)
    gh_token = gh_token if gh_token != "" else None

    if args.subcommand == "maint-nightly":
        return cmd_maint_nightly(
            config, log, dry_run=args.dry_run, gh_token=gh_token)
    else:
        log.critical(f"Unhandled subcommand: {args.subcommand}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
