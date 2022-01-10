import json
import csv
import os
import shutil
import re


def main():
    external_projects = [load_grant_projects(),
                         load_icp123_projects(),
                         load_icapp_projects(),
                         load_showcase_projects()]

    # load projects from csv file
    file_path = 'reserved.json'
    projects = []
    if os.path.exists(file_path):
        # backup old file
        shutil.copy(file_path, 'reserved.json.bak')
        with open(file_path, 'r', encoding='utf8') as f:
            projects = [
                Project(p["id"], p["title"], p["summary"], p["grant"], p["icnaming"], p["recommend_url"], p["type"],
                        p["url"]) for p
                in
                json.load(f)]
        print(f"csv loaded from {file_path}")
    else:
        print(f"{file_path} not found")

    # add diff project from json to csv
    # grantee as key
    for ext_projects in external_projects:
        for p in ext_projects:
            if p.id not in [p.id for p in projects]:
                projects.append(p)

    print(f"{len(projects)} projects loaded")

    # load reserved.txt
    reserved_path = 'reserved.txt'
    reserved = []
    if os.path.exists(reserved_path):
        with open(reserved_path, 'r', encoding='utf8') as f:
            for line in f:
                if line.strip():
                    reserved.append(line.strip())
        for res in reserved:
            if res not in [p.icnaming for p in projects]:
                projects.append(Project(res, "", "", "", res, "", "reserved", ""))

    # sort projects by grantee then by icnaming
    projects.sort(key=lambda p: (p.id, p.icnaming))

    # update icnaming as lower of id
    for i, p in enumerate(projects):
        p.recommend_url = p.id.lower().replace(" ", "-")

    # write projects to csv file
    with open('reserved.csv', 'w', newline='', encoding='utf8') as f:
        writer = csv.writer(f)
        writer.writerow(['id', 'title', 'summary', 'grant', 'icnaming', 'recommend_url', 'type', "url"])
        for p in projects:
            writer.writerow([p.id, p.title, p.summary, p.grant, p.icnaming, p.recommend_url, p.type, p.url])

    print(f"{len(projects)} projects written to {file_path}")

    # write projects to json file
    with open(file_path, 'w', encoding='utf8') as f:
        json.dump([p.serialize() for p in projects], f, indent=4)

    create_reserved_list()


def load_grant_projects():
    json_path = 'projects/grant_20220107.json'
    # load projects from json file
    with open(json_path, 'r', encoding='utf8') as f:
        projects = json.load(f)
    new_projects = [Project(p["grantee"].strip().lower(), p["grantee"], p["summary"], p["grant"], "", "", "grant", "")
                    for p
                    in
                    projects]

    print(f"json loaded from {json_path}")

    return new_projects


def load_icp123_projects():
    json_path = 'projects/icp123_20220109.json'
    # load projects from json file
    with open(json_path, 'r', encoding='utf8') as f:
        projects = json.load(f)
    new_projects = [Project(p["id"].strip().lower(), p["title"], p["summary"], "", "", "", "icp123", p["url"]) for p in
                    projects['pageProps']['projectData']]

    print(f"json loaded from {json_path}")

    return new_projects


def load_icapp_projects():
    json_path = 'projects/icapp_20220109.json'
    # load projects from json file
    with open(json_path, 'r', encoding='utf8') as f:
        projects = json.load(f)
    new_projects = [Project(p[0].strip().lower(), p[1], p[19], "", "", "", "icapp", p[5]) for p in
                    projects['valueRanges'][0]['values'][1:]]

    print(f"json loaded from {json_path}")

    return new_projects


def load_showcase_projects():
    json_path = 'projects/showcase_20220109.json'
    # load projects from json file
    with open(json_path, 'r', encoding='utf8') as f:
        projects = json.load(f)
    new_projects = [
        Project(p["name"].strip().lower(), p["name"], p["description"]["childMarkdownRemark"]["html"], "", "", "",
                "showcase", p["website"]) for p in
        projects['result']['data']['page']['projects']]

    print(f"json loaded from {json_path}")

    return new_projects


class Project:
    def __init__(self, id, title, summary, grant, icnaming, recommend_url, type, url):
        self.id = id
        self.title = title
        self.summary = summary
        self.grant = str(grant)
        self.icnaming = icnaming
        self.type = type
        self.url = url
        self.recommend_url = recommend_url

    def serialize(self):
        """Return object data in easily serializeable format"""
        return {
            'id': self.id,
            'title': self.title,
            'summary': self.summary,
            'grant': self.grant,
            'icnaming': self.icnaming,
            'type': self.type,
            'url': self.url,
            'recommend_url': self.recommend_url
        }


def csv_to_json():
    csv_path = 'reserved.csv'
    json_path = 'reserved.json'
    # load projects from csv file
    with open(csv_path, 'r', encoding='utf8') as f:
        reader = csv.reader(f)
        projects = list(reader)
    new_projects = [Project(p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]) for p in projects[1:]]

    print(f"csv loaded from {csv_path}")

    # write projects to json file
    with open(json_path, 'w', encoding='utf8') as f:
        json.dump([p.serialize() for p in new_projects], f, indent=4)


def create_reserved_list():
    json_path = 'reserved.json'
    # load projects from json file
    with open(json_path, 'r', encoding='utf8') as f:
        projects = json.load(f)

    # create map of reserved projects with icnaming as key
    reserved_projects = {}
    for p in projects:
        if p['icnaming'] not in reserved_projects:
            reserved_projects[p['icnaming']] = []
        reserved_projects[p['icnaming']].append(p['id'])

    result = {}
    for k, v in reserved_projects.items():
        result[k] = v[0]
        kk = k.replace('-', '')
        if kk != k:
            result[kk] = v[0]

    # remove 'tbd' and 'skip'
    result.pop('tbd', None)
    result.pop('skip', None)

    result = sorted(result.items())

    # ensure all keys are contains only lowercase letters, numbers and '-'
    for k, v in result:
        if not re.match('^[a-z0-9-]+$', k):
            print(f"{k} is not a valid key")
            exit(1)

    # write result to csv file
    with open('reserved_list.csv', 'w', encoding='utf8', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['icnaming', 'id'])
        # result to array order by icnaming
        for k, v in result:
            writer.writerow([k, v])

    # write keys to rs file
    with open('../canisters/registrar/src/reserved_list.rs', 'w', encoding='utf8') as f:
        f.write("pub const RESERVED_NAMES: &[&str] = &[\n")
        for k, v in result:
            f.write(f"    \"{k}\",\n")
        f.write("];\n")
        
    print(f"reserved list created")


if __name__ == "__main__":
    main()
    # csv_to_json()
