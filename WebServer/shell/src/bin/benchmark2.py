from locust import HttpUser, task

class WebsiteUser(HttpUser):
    @task
    def f100000(self):
        self.client.get("/file100000.html")

    @task
    def f1000000(self):
        self.client.get("/file1000000.html")