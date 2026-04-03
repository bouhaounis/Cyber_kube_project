import { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { useClusterStore } from '../../stores/clusterStore';

export function Cluster3D() {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const animationIdRef = useRef<number | null>(null);
  const { pods, selectPod } = useClusterStore();

  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize scene
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a0a0a);
    sceneRef.current = scene;

    // Camera
    const camera = new THREE.PerspectiveCamera(
      75,
      containerRef.current.clientWidth / containerRef.current.clientHeight,
      0.1,
      1000
    );
    camera.position.set(0, 5, 10);
    camera.lookAt(0, 0, 0);
    cameraRef.current = camera;

    // Renderer
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    containerRef.current.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // Lighting
    const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(10, 10, 5);
    scene.add(directionalLight);

    // Create nodes (representing Kubernetes nodes)
    const nodeGeometry = new THREE.SphereGeometry(0.3, 16, 16);
    const nodeMaterial = new THREE.MeshPhongMaterial({ color: 0x3b82f6 });
    const nodes: THREE.Mesh[] = [];

    for (let i = 0; i < 3; i++) {
      const node = new THREE.Mesh(nodeGeometry, nodeMaterial.clone());
      const angle = (i / 3) * Math.PI * 2;
      node.position.set(
        Math.cos(angle) * 3,
        0,
        Math.sin(angle) * 3
      );
      scene.add(node);
      nodes.push(node);
    }

    // Create pods (smaller spheres around nodes)
    const podGeometry = new THREE.SphereGeometry(0.15, 12, 12);
    const podMeshes: THREE.Mesh[] = [];

    const podCount = pods.length || 8;
    for (let i = 0; i < podCount; i++) {
      const pod = new THREE.Mesh(
        podGeometry,
        new THREE.MeshPhongMaterial({
          color: i % 2 === 0 ? 0x10b981 : 0xf59e0b,
        })
      );
      const nodeIndex = i % nodes.length;
      const node = nodes[nodeIndex];
      const angle = (i / podCount) * Math.PI * 2;
      pod.position.set(
        node.position.x + Math.cos(angle) * 0.8,
        node.position.y + 0.5 + (i % 3) * 0.3,
        node.position.z + Math.sin(angle) * 0.8
      );
      scene.add(pod);
      podMeshes.push(pod);

      // Make pods clickable
      pod.userData = { podIndex: i };
    }

    // Connections between nodes
    const lineMaterial = new THREE.LineBasicMaterial({ color: 0x4b5563, opacity: 0.3, transparent: true });
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const geometry = new THREE.BufferGeometry().setFromPoints([
          nodes[i].position,
          nodes[j].position,
        ]);
        const line = new THREE.Line(geometry, lineMaterial);
        scene.add(line);
      }
    }

    // Mouse interaction
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    const onMouseMove = (event: MouseEvent) => {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    };

    const onMouseClick = () => {
      raycaster.setFromCamera(mouse, camera);
      const intersects = raycaster.intersectObjects(podMeshes);
      if (intersects.length > 0) {
        const podIndex = intersects[0].object.userData.podIndex;
        if (pods[podIndex]) {
          selectPod(pods[podIndex]);
        }
      }
    };

    containerRef.current.addEventListener('mousemove', onMouseMove);
    containerRef.current.addEventListener('click', onMouseClick);

    // Orbit controls simulation (simple rotation)
    let angle = 0;
    const animate = () => {
      angle += 0.005;
      camera.position.x = Math.cos(angle) * 10;
      camera.position.z = Math.sin(angle) * 10;
      camera.lookAt(0, 0, 0);

      // Rotate pods slightly
      podMeshes.forEach((pod, i) => {
        pod.rotation.y += 0.01;
        pod.position.y += Math.sin(Date.now() * 0.001 + i) * 0.001;
      });

      renderer.render(scene, camera);
      animationIdRef.current = requestAnimationFrame(animate);
    };
    animate();

    // Handle resize
    const handleResize = () => {
      if (!containerRef.current || !camera || !renderer) return;
      camera.aspect = containerRef.current.clientWidth / containerRef.current.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
    };
    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      containerRef.current?.removeEventListener('mousemove', onMouseMove);
      containerRef.current?.removeEventListener('click', onMouseClick);
      if (animationIdRef.current) {
        cancelAnimationFrame(animationIdRef.current);
      }
      if (rendererRef.current && containerRef.current) {
        containerRef.current.removeChild(rendererRef.current.domElement);
      }
      rendererRef.current?.dispose();
    };
  }, [pods, selectPod]);

  return <div ref={containerRef} className="w-full h-full" />;
}
