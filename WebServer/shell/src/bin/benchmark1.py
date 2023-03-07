from locust import HttpUser, task

class WebsiteUser(HttpUser):
    @task
    def f100(self):
        self.client.get("/file100.html")

    @task
    def f1000(self):
        self.client.get("/file1000.html")