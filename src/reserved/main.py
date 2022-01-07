import json
import csv
import os
import shutil


def main():
    json_path = 'projects/20220107.json'
    # load projects from json file
    with open(json_path, 'r') as f:
        projects = json.load(f)
    new_projects = [Project(p["grantee"], p["summary"], p["grant"], "", "grant") for p in projects]

    print(f"json loaded from {json_path}")

    # load projects from csv file
    file_path = 'projects.csv'
    projects = []
    if os.path.exists(file_path):
        # backup old file
        shutil.copy(file_path, 'projects.csv.bak')
        with open(file_path, 'r') as f:
            reader = csv.reader(f)
            # skip header
            next(reader)
            for row in reader:
                # map row to project
                project = Project(row[0], row[1], row[2], row[3] if len(row) > 3 else "",
                                  row[4] if len(row) > 4 else "")
                projects.append(project)
        print(f"csv loaded from {file_path}")
    else:
        print(f"{file_path} not found")

    # add diff project from json to csv
    # grantee as key
    for p in new_projects:
        if p.grantee not in [p.grantee for p in projects]:
            projects.append(p)

    print(f"{len(projects)} projects loaded")

    # load reserved.txt
    reserved_path = 'reserved.txt'
    reserved = []
    if os.path.exists(reserved_path):
        with open(reserved_path, 'r') as f:
            for line in f:
                if line.strip():
                    reserved.append(line.strip())
        for res in reserved:
            if res not in [p.icnaming for p in projects]:
                projects.append(Project("", "", "", res, "reserved"))

    # sort projects by grantee then by icnaming
    projects.sort(key=lambda p: (p.grantee, p.icnaming))

    # write projects to csv file
    with open('projects.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['grantee', 'summary', 'grant', 'icnaming', 'type'])
        for p in projects:
            writer.writerow([p.grantee, p.summary, p.grant, p.icnaming, p.type])

    print(f"{len(projects)} projects written to {file_path}")


class Project:
    def __init__(self, grantee, summary, grant, icnaming, type):
        self.grantee = grantee
        self.summary = summary
        self.grant = grant
        self.icnaming = icnaming
        self.type = type


if __name__ == "__main__":
    main()
