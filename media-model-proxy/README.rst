.. image:: ../../_static/media_model_proxy_service.png

.. _service-media-model-proxy:

Media Model Proxy (MMP)
-----------------------
Media Model Proxy provides a proxy service to the DeepBird Prediction services
operated by Media Understanding.

The service intermediates with Blobstore to load the media requested and can
issue inference requests to multiple models simultaneously. This is useful to
reduce Blobstore and network usage for clients. Clients themselves can pass
only lightweight metadata (model flags, media type, Blobstore path, etc.).

Media Model Proxy does not publish/store model scores for consumption, instead
it returns a map of the requested models to the observed responses to the
caller. The caller is responsible with interpreting and persisting the results.

Current clients are Media Analysis Service. Periscope issues requests through a
custom thrift endpoint in Media Analysis Service.

(This service was originally Cortex Media Annotator. Design Documentation and
service notes regarding Cortex Media Annotator are potentially also of use.)

* Runbook: `the internal runbook <https://example.invalid/mediamodelproxygrunbook>`_

Media Model Proxy source code is at `media-understanding/media-model-proxy`_
in Source.

.. _`media-understanding/media-model-proxy`: https://example.invalid/code/media-understanding/media-model-proxy

Contents:

.. toctree::
  :maxdepth: 3

  using
  service
